mod shell;
mod model;

use std::collections::HashMap;
use std::io::{self, Write};
use std::fs;
use std::process::Command as ProcessCommand;
use serde::{Deserialize, Serialize};
use clap::{Command, Arg};
use colored::*;
use std::path::PathBuf;
use shell::Shell;
use crate::model::Model;

#[derive(Serialize, Deserialize)]
struct Config {
    model: Model,
    max_tokens: i32
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = Command::new("llm-term")
        .version("1.0")
        .author("dh1101")
        .about("Generate terminal commands using OpenAI or local Ollama models")
        .arg(Arg::new("prompt")
            .help("The prompt describing the desired command")
            .required(false)
            .index(1))
        .arg(Arg::new("config")
            .short('c')
            .long("config")
            .help("Run configuration setup")
            .action(clap::ArgAction::SetTrue))
        .arg(
            Arg::new("disable-cache")
                .long("disable-cache")
                .help("Disable cache and always query the LLM")
                .action(clap::ArgAction::SetTrue),
        )
        .get_matches();

    let config_path = get_default_config_path().expect("Failed to get default config path");

    if matches.get_flag("config") {
        let config = create_config()?;
        let content = serde_json::to_string_pretty(&config)?;
        fs::write(&config_path, content)?;
        println!("{}", "Configuration saved successfully.".green());
        return Ok(());
    }

    let config = load_or_create_config(&config_path)?;

    let cache_path = get_cache_path()?;
    let mut cache = load_cache(&cache_path)?;

    if let Some(prompt) = matches.get_one::<String>("prompt") {
        let disable_cache = matches.get_flag("disable-cache");

        if !disable_cache {
            if let Some(cached_command) = cache.get(prompt) {
                println!("{}", "This command exists in cache".yellow());
                println!("{}", cached_command.cyan().bold());
                println!("{}", "Do you want to execute this command? (y/n)".yellow());

                let mut user_input = String::new();
                io::stdin().read_line(&mut user_input)?;

                if user_input.trim().to_lowercase() == "y" {
                    execute_command(cached_command)?;
                } else {
                    println!("{}", "Do you want to invalidate the cache? (y/n)".yellow());
                    user_input.clear();
                    io::stdin().read_line(&mut user_input)?;

                    if user_input.trim().to_lowercase() == "y" {
                        // Invalidate cache
                        cache.remove(prompt);
                        save_cache(&cache_path, &cache)?;
                        // Proceed to get command from LLM
                        let _ = get_command_from_llm(&config, &mut cache, &cache_path, prompt).await;
                    } else {
                        println!("{}", "Command execution cancelled.".yellow());
                    }
                }
                return Ok(());
            } else {
                // Not in cache, proceed to get command from LLM
                let _ = get_command_from_llm(&config, &mut cache, &cache_path, prompt).await;
            }
        } else {
            // Cache is disabled, proceed to get command from LLM
            let _ = get_command_from_llm(&config, &mut cache, &cache_path, prompt).await;
        }
    } else {
        println!("{}", "Please provide a prompt or use --config to set up the configuration.".yellow());
    }

    Ok(())
}

fn get_default_config_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let exe_path = std::env::current_exe()?;
    let exe_dir = exe_path.parent().ok_or("Failed to get executable directory")?;
    Ok(exe_dir.join("config.json"))
}

fn load_or_create_config(path: &PathBuf) -> Result<Config, Box<dyn std::error::Error>> {
    if let Ok(content) = fs::read_to_string(path) {
        Ok(serde_json::from_str(&content)?)
    } else {
        let config = create_config()?;
        let content = serde_json::to_string_pretty(&config)?;
        fs::write(path, content)?;
        Ok(config)
    }
}

fn create_config() -> Result<Config, io::Error> {
    let model = loop {
        println!("{}", "Select model:\n 1 for gpt-4o-mini\n 2 for gpt-4o\n 3 for ollama (qwen2.5-coder)".cyan());

        io::stdout().flush()?;
        let mut choice = String::new();
        io::stdin().read_line(&mut choice)?;
        match choice.trim() {
            "1" => break Model::OpenAiGpt4oMini,
            "2" => break Model::OpenAiGpt4o,
            "3" => break Model::Ollama("qwen2.5-coder".to_string()),
            _ => println!("{}", "Invalid choice. Please try again.".red()),
        }
    };

    let max_tokens = loop {
        print!("{}", "Enter max tokens (1-4096): ".cyan());
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        if let Ok(tokens) = input.trim().parse::<i32>() {
            if tokens > 0 && tokens <= 4096 {
                break tokens;
            }
        }
        println!("{}", "Invalid input. Please enter a number between 1 and 4096.".red());
    };

    Ok(Config {
        model,
        max_tokens,
    })
}

fn get_cache_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let exe_path = std::env::current_exe()?;
    let exe_dir = exe_path.parent().ok_or("Failed to get executable directory")?;
    Ok(exe_dir.join("cache.json"))
}

fn load_cache(path: &PathBuf) -> Result<HashMap<String, String>, Box<dyn std::error::Error>> {
    if let Ok(content) = fs::read_to_string(path) {
        Ok(serde_json::from_str(&content)?)
    } else {
        Ok(HashMap::new())
    }
}

fn save_cache(path: &PathBuf, cache: &HashMap<String, String>) -> Result<(), Box<dyn std::error::Error>> {
    let content = serde_json::to_string_pretty(&cache)?;
    fs::write(path, content)?;
    Ok(())
}

async fn get_command_from_llm(
    config: &Config,
    cache: &mut HashMap<String, String>,
    cache_path: &PathBuf,
    prompt: &String,
) -> Result<(), Box<dyn std::error::Error>> {
    match &config.model.llm_get_command(config, prompt.as_str()).await {
        Ok(Some(command)) => {
            println!("{}", &command.cyan().bold());
            //println!("{}", format!("Generated command: {}", command).green());
            println!("{}", "Do you want to execute this command? (y/n)".yellow());

            let mut user_input = String::new();
            io::stdin().read_line(&mut user_input)?;

            if user_input.trim().to_lowercase() == "y" {
                execute_command(&command)?;
            } else {
                println!("{}", "Command execution cancelled.".yellow());
            }

            // Save command to cache
            cache.insert(prompt.clone(), command.clone());
            save_cache(cache_path, cache)?;
        },
        Ok(None) => println!("{}", "No command could be generated.".yellow()),
        Err(e) => eprintln!("{}", format!("Error: {}", e).red()),
    }

    Ok(())
}

fn execute_command(command: &str) -> Result<(), Box<dyn std::error::Error>> {
    let (shell_cmd, shell_arg) = Shell::detect().to_shell_command_and_command_arg();

    match ProcessCommand::new(shell_cmd).arg(shell_arg).arg(&command).output() {
        Ok(output) => {
            println!("{}", "Command output:".green().bold());
            io::stdout().write_all(&output.stdout)?;
            io::stderr().write_all(&output.stderr)?;
        }
        Err(e) => eprintln!("{}", format!("Failed to execute command: {}", e).red()),
    }

    Ok(())
}

use openai_api_rust::{Auth, Message, OpenAI, Role};
use openai_api_rust::chat::{ChatApi, ChatBody};
use serde::{Deserialize, Serialize};
use crate::Config;
use crate::shell::Shell;
use reqwest::Client;
use serde_json::json;

#[derive(Serialize, Deserialize)]
pub enum Model {
    #[serde(rename = "gpt-4o")]
    OpenAiGpt4o,

    #[serde(rename = "gpt-4o-mini")]
    OpenAiGpt4oMini,

    #[serde(rename = "ollama")]
    Ollama(String),
}

impl Model {
    pub async fn llm_get_command(&self, config: &Config, user_prompt: &str) -> Result<Option<String>, Box<dyn std::error::Error>> {
        match self {
            Model::Ollama(model_name) => {
                let client = Client::new();
                let system_prompt = self.get_system_prompt(&Shell::detect());
                let full_prompt = format!("{}{}", system_prompt, user_prompt);
                let body = json!({
                    "model": model_name,
                    "messages": [{"role": "user", "content": full_prompt}],
                    "stream": false,
                });

                let response = client
                    .post("http://localhost:11434/api/chat")
                    .json(&body)
                    .send()
                    .await?;

                let response_body = response.text().await?;
                //println!("{}", format!("Full Ollama response: {}", response_body).green());
                let json_response: serde_json::Value = serde_json::from_str(&response_body)?;

                if let Some(message) = json_response.get("message") {
                    if let Some(content) = message.get("content").and_then(|v| v.as_str()) {
                        Ok(Some(content.to_string()))
                    } else {
                        Ok(None)
                    }
                } else {
                    Ok(None)
                }
            }
            _ => {
                let model_name = self.get_model_name();
                let auth = self.get_auth();
                let client = OpenAI::new(auth, self.get_openai_endpoint().as_str());

                let shell = Shell::detect();
                let system_prompt = self.get_system_prompt(&shell);

                let body = ChatBody {
                    model: model_name,
                    max_tokens: Some(config.max_tokens),
                    temperature: Some(0.5),
                    top_p: None,
                    n: None,
                    stream: None,
                    stop: None,
                    presence_penalty: None,
                    frequency_penalty: None,
                    logit_bias: None,
                    user: None,
                    messages: vec![
                        Message { role: Role::System, content: system_prompt.to_string() },
                        Message { role: Role::User, content: user_prompt.to_string() }
                    ],
                };

                match client.chat_completion_create(&body) {
                    Ok(response) => Ok(response.choices.first()
                        .map(|choice| choice.message.as_ref())
                        .flatten()
                        .map(|message| message.content.clone())
                    ),
                    Err(e) => Err(format!("Error: {:?}", e).into()),
                }
            }
        }
    }

    fn get_model_name(&self) -> String {
        match self {
            Model::OpenAiGpt4o => "gpt-4o".to_string(),
            Model::OpenAiGpt4oMini => "gpt-4o-mini".to_string(),
            Model::Ollama(model_name) => model_name.to_string(),
        }
    }

    fn get_openai_endpoint(&self) -> String {
        match self {
            Model::OpenAiGpt4o => "https://api.openai.com/v1/".to_string(),
            Model::OpenAiGpt4oMini => "https://api.openai.com/v1/".to_string(),
            Model::Ollama(_) => "http://localhost:11434/api/chat".to_string(),
        }
    }

    fn get_auth(&self) -> Auth {
        match self {
            Model::OpenAiGpt4o => Auth::from_env().expect("OPENAI_API_KEY environment variable not set"),
            Model::OpenAiGpt4oMini => Auth::from_env().expect("OPENAI_API_KEY environment variable not set"),
            Model::Ollama(_) => Auth::new("ollama"),
        }
    }

    /// Generates the LLM system prompt for the shell.
    fn get_system_prompt(&self, shell: &Shell) -> String {
        let shell_command_type = match shell {
            Shell::Powershell => "Windows PowerShell",
            Shell::BornAgainShell => "Bourne AgainShell (bash / sh)",
            Shell::Zsh => "Z Shell (zsh)",
            Shell::Fish => "Friendly Interactive Shell (fish)",
            Shell::DebianAlmquistShell => "Debian Almquist Shell (dash)",
            Shell::KornShell => "Korn Shell (ksh)",
            Shell::CShell => "C Shell (csh)",
            Shell::Unknown => "",
        };

        format!("You are a professional IT automation specialist who translates user requests into precise CLI commands. Respond ONLY with a single valid {} command compatible with {} operating system. Output must be the raw command text without any formatting, backticks, code blocks, or explanations.

Critical Rules:
1. **Never use** markdown, code fences (```), quotes, or any syntax
2. **Never add** comments, notes, or multiple commands
3. **Always verify** command validity before responding
4. **Only output** one executable command per response
5. **If uncertain**, return empty string

Example GOOD output: ls -la
Example BAD output: ```sh\nls -la\n```

The command must run as-is in the user's current directory. Prioritize safety and accuracy above all.", shell_command_type, std::env::consts::OS)
    }
}

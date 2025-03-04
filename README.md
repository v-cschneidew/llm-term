# 🖥️ LLM-Term

A Rust-based CLI tool that generates and executes terminal commands using OpenAI's language models or local Ollama models.

## Features

- Configurable model and token limit (gpt-4o-mini, gpt-4o, or Ollama)
- Generate and execute terminal commands based on user prompts
- Works on both PowerShell and Unix-like shells (Automatically detected)

## Installation

**Prerequisites:**

- [Rust](https://www.rust-lang.org/tools/install)

**Build Instructions:**

1.  Clone the repository:

    ```bash
    git clone https://github.com/dh1011/llm-term.git
    cd llm-term
    ```

2.  Build the project using Cargo:

    ```bash
    cargo build --release
    ```

3.  The executable will be available in the `target/release` directory.

**Setting the PATH:**

To make the `llm-term` executable accessible from any terminal, you need to add it to your shell's PATH environment variable.

- **macOS/Linux:**

  ```bash
  export PATH="$HOME/llm-term/target/release:$PATH"
  ```

  To make this permanent, add the above line to your shell configuration file (e.g., `~/.bashrc`, `~/.zshrc`).

  You can also add an alias to your shell configuration file (e.g., `~/.bashrc`, `~/.zshrc`) for easier access:

  ```bash
  unsetopt interactivecomments
  alias '#'='llm-term'
  ```

- **Windows:**

  ```powershell
  $env:PATH += ";C:\path\to\llm-term\target\release"
  ```

  To make this permanent, add the above line to your PowerShell profile (e.g., `$PROFILE`). You may need to create the profile if it doesn't exist.

## Usage

1.  Set your OpenAI API key (if using OpenAI models):

    - macOS/Linux:

      ```bash
      export OPENAI_API_KEY="sk-..."
      ```

    - Windows:

      ```powershell
      $env:OPENAI_API_KEY="sk-..."
      ```

2.  If using Ollama, make sure it's running locally on the default port (11434).

3.  Run the application with a prompt:

    ```bash
    llm-term your prompt here
    ```

    or, if you set up the alias:

    ```bash
    # your prompt here
    ```

4.  The app will generate a command based on your prompt and ask for confirmation before execution.

## Configuration

A `config.json` file will be created in the same directory as the binary on first run. You can modify this file to change the default model and token limit.

## Options

- `-c, --config <FILE>`: Specify a custom config file path

## Supported Models

- OpenAI GPT-4 (gpt-4o) (Untested)
- OpenAI GPT-4 Mini (gpt-4o-mini) (Untested)
- Ollama (local models, default: qwen2.5-coder)

# chatgptr

A TUI interface for ChatGPT, written in Rust.

## Features

- Converse with ChatGPT
- Specify which model to use

### Roadmap

- Continue with conversations
- Beautiful TUI interface
- Browse through conversations
- Save conversations in plaintext format

## Installation

### Prepackaged binaries

There are prepackaged binaries available to download from the [releases page](/releases), for Linux, MacOS, and Windows.

### Cargo (via crates.io)

chatgptr can be installed from crates.io using cargo:

```
cargo install chatgptr
```

### Build from source

If you want to build chatgptr from the source code, you can `git clone` this repository and run:

```
cargo build -r
```

## Configuration

`$XDG_CONFIG_HOME/.config/chatgptr/chatgptr.toml`

```
token=<your OpenAI API token>
model=<GPT model to use>
```
You can obtain an API key from [OpenAI's website](https://platform.openai.com/api-keys) (requires an account and credit card information).

GPT model has to take the format as specified [by OpenAI](https://platform.openai.com/docs/models).

## Usage

To use the CLI binary, use the `chatgpt` command. It supports several subcommands:

- `chatgpt ask <question>`: sends `<question>` to API and outputs the answer
- `chatgpt help`: shows usage
- `chatgpt version`: outputs the version of chatgptr
- `chatgpt author`: outputs the author of chatgptr
- `chatgpt config`: outputs location of config file

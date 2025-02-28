NB: this README was generated from the project itself.

# Summarize

A Rust CLI tool that uses AI to generate comprehensive README documentation for codebases by analyzing source files.

## Overview

Summarize is a developer tool that solves the problem of understanding and documenting codebases with minimal effort. It scans your project's source files, sends them to an AI model (by default GPT-4o-mini), and generates a detailed README that helps developers quickly understand the project structure, key abstractions, and usage patterns.

## Features

- Scan source files in a directory based on file extensions or glob patterns
- Stream file contents to handle large codebases without memory issues
- Generate comprehensive documentation using AI that understands code context
- Customizable file selection to focus on the most relevant parts of your codebase
- Support for different AI models

## Installation

### From Source

To build from source, ensure you have Rust installed (2024 edition), then:

```bash
git clone https://github.com/yourusername/summarize.git
cd summarize
cargo build --release
```

The binary will be available at `./target/release/summarize`.

## Usage

```bash
# Generate README for current directory
summarize

# Specify a directory to analyze
summarize --dir /path/to/project

# Only include specific file types
summarize --file-types rs js ts

# Use glob patterns to include specific files
summarize --globs "src/**/*.rs" "lib/**/*.rs"

# Use a different AI model
summarize --model gpt-4o-mini
```

### Command-line Options

```
OPTIONS:
    -d, --dir <DIR>                  The directory to walk. Defaults to the current dir
    -f, --file-types <FILE_TYPES>    The file types to include (e.g. 'kt', 'rs')
    -g, --globs <GLOBS>              Globs to include
    -m, --model <MODEL>              AI model to use [default: gpt-4o-mini]
    -h, --help                       Print help
```

## How It Works

Summarize works in three main steps:

1. **File Collection**: The tool scans the specified directory for source files that match the given file types or glob patterns.

```rust
let mut stream = files::stream(FindOpts {
    dir: dir.clone(),
    file_types: args.file_types.clone(),
    globs: args.globs.clone(),
});
```

2. **File Processing**: It reads each file and builds a prompt containing the file contents.

```rust
while let Some(res) = stream.next().await {
    let info = res?;
    let path = info.path.to_string_lossy();
    let contents = String::from_utf8_lossy(&info.bs).to_string();
    buf.push_str(&header);
    buf.push_str(&format!("\n## Path:{}\n\n{}\n", path, contents));
}
```

3. **AI Generation**: The collected file contents are sent to the AI model, which analyzes the code and generates a comprehensive README.

```rust
let client = Client::default();
let resp: ChatResponse = client
    .exec_chat(model, reqs, None)
    .await
    .context("failed to call model")?;
```

## Example

Here's an example of generating a README for a Rust project:

```bash
$ cd /path/to/my/rust/project
$ summarize --file-types rs --globs "src/**/*.rs"
```

The tool will scan all Rust files in the src directory, analyze them, and output a comprehensive README to the console, which you can then save to your project.

## Code Structure

- `main.rs`: Entry point that parses command-line arguments and runs the main function
- `lib.rs`: Core logic including argument definitions, AI client setup, and main processing loop
- `files.rs`: File handling utilities for finding and streaming file contents

## Dependencies

The project relies on several key dependencies:

- **clap**: Command-line argument parsing
- **genai**: Client for interacting with AI models
- **ignore**: Fast file system traversal (similar to ripgrep)
- **tokio**: Asynchronous runtime for efficient file processing
- **futures-util**: Utilities for working with asynchronous streams

## Next Steps for Contributors

If you're interested in contributing to this project, here are some good places to start:

1. **Prompt Engineering**: Improve the prompt template in `prompt.md` to generate better READMEs
2. **File Filtering**: Enhance the file selection logic to better handle different project structures
3. **Output Options**: Add support for writing output to a file and other formatting options
4. **Model Selection**: Implement support for additional AI models and providers
5. **Performance**: Optimize file processing for very large codebases

## License

[License Information]

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

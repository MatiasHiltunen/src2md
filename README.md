# src2md

src2md is a command-line tool written in Rust that traverses a project directory, collects code and text files, and compiles them into a single Markdown (.md) file. It respects ignore files (like .gitignore) and allows for customization through command-line options.

## Features

- Recursively scans directories to find files.
- Supports custom ignore files, defaulting to .src2md.ignore or .gitignore.
- Option to include specific files or directories.
- Wraps code in Markdown code blocks with appropriate language tags for syntax highlighting.
- Lists binary files by their paths without including content.
- Zero-Copy File Reading, uses memory-mapped files for efficient file reading.


## Installation

### Prerequisites

Ensure you have Rust and Cargo installed.

To install src2md with cargo, run command:
```sh
cargo install --git https://github.com/MatiasHiltunen/src2md.git
```

### Build from Source

Clone the repository and build the project:
```sh
git clone https://github.com/yourusername/src2md.git
cd src2md
cargo build --release
```
  
This will create an executable in target/release/src2md.

## Usage

Run the src2md executable with various options:

./target/release/src2md [OPTIONS] [PATHS]...

### Command-Line Options

-o, --output <FILE>: Sets the output .md file path. Defaults to all_the_code.md in the current directory.

-i, --ignore <FILE>: Sets the ignore file path. If not specified, it tries to use .src2md.ignore or .gitignore.

[PATHS]: Specific files or directories to include. If provided, only these paths are processed.


### Examples

Default Usage

Collect all code and text files in the current directory and output to all_the_code.md:
```sh
./target/release/src2md
```

Specify Output File Path

Output to a custom file path:
```sh
./target/release/src2md -o docs/all_code.md
```
Use Custom Ignore File

Use a custom ignore file instead of .src2md.ignore or .gitignore:
```sh
./target/release/src2md -i custom.ignore
```
Include Specific Files or Directories

Process only specific files or directories:
```sh
./target/release/src2md src/main.rs src/lib.rs
```

Combine Options

Combine multiple options:
```sh
./target/release/src2md -o code.md -i custom.ignore src/ tests/test_main.rs
```

The ignore file follows the same syntax as .gitignore.

### Handling Multiple Runs

By default, src2md overwrites the output file each time it runs. If you need to append to the existing file or keep backups, consider modifying the file handling logic in the code or incorporate versioning in your workflow.

## Contributing

Contributions are welcome! Feel free to open issues or submit pull requests on GitHub.

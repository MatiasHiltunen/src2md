# src2md

[![crates.io](https://img.shields.io/crates/v/src2md.svg)](https://crates.io/crates/src2md)
[![docs.rs](https://docs.rs/src2md/badge.svg)](https://docs.rs/src2md)
[![license](https://img.shields.io/crates/l/src2md.svg)](https://github.com/MatiasHiltunen/src2md/blob/main/LICENSE)

Turn source/text files into a single Markdown document — or restore them back. Built with Rust **2024 edition**.

> **Warning**  
> This project is in early development and may change rapidly.  
> AI (ChatGPT) has been extensively used in the design and implementation of this codebase.  
> Review carefully before using in production or contributing.

---

## Features

- Recursively scans directories to find files.
- **Automatically excludes** hidden files, lock files, and previous outputs.
- Supports `.src2md.ignore`, `.gitignore`, or a custom ignore file.
- **Filter by file extension** with `--ext` flag.
- Option to include specific files or directories.
- Wraps content in Markdown code blocks with syntax highlighting.
- Uses dynamic backtick fencing to safely include Markdown and code.
- Lists binary files by their paths (content omitted).
- **Zero-copy** file reading using memory-mapped files.
- Restore files back from a generated Markdown file (with `--restore`).
- Usable as a **command-line tool** or **Rust library**.

---

## Installation

```bash
cargo install src2md
```

Or build from source:

```bash
git clone https://github.com/MatiasHiltunen/src2md.git
cd src2md
cargo build --release
```

---

## CLI Usage

```bash
src2md [OPTIONS] [PATHS]...
```

### Common Options

| Flag                   | Description                                                        |
|------------------------|--------------------------------------------------------------------|
| `-o, --output FILE`    | Output Markdown file (default: `{project}_content_{timestamp}.md`) |
| `--ignore-file FILE`   | Path to ignore file (like `.gitignore`)                            |
| `-e, --ext EXTENSIONS` | Only include files with these extensions (comma-separated)         |
| `-v, --verbose`        | Enable verbose output (repeat for more: `-v`, `-vv`, `-vvv`)       |
| `[PATHS]`              | Files or directories to include                                    |
| `--restore FILE.md`    | Restore files from a src2md Markdown file back to filesystem       |
| `--restore-path DIR`   | Target directory to restore files into                             |
| `--fail-fast`          | Stop processing on first error                                     |

### Default Exclusions

The following are **always excluded** by default:

- **Hidden files and directories** (starting with `.`)
- **Lock files**: `package-lock.json`, `yarn.lock`, `pnpm-lock.yaml`, `Cargo.lock`, `Gemfile.lock`, `poetry.lock`, `Pipfile.lock`, `composer.lock`, `mix.lock`, `pubspec.lock`, `flake.lock`, `go.sum`, and any `*.lock` file
- **Previous src2md output files** (detected by magic header)

### Examples

```bash
# Default: all files in current dir → Markdown
src2md

# Specify output path
src2md -o docs/code.md

# Use custom ignore file
src2md --ignore-file .customignore

# Include only Rust and TypeScript files
src2md --ext rs,ts

# Include only certain files
src2md src/lib.rs src/main.rs

# Verbose output showing what's being processed
src2md -vv -o output.md

# Restore files from Markdown back to filesystem
src2md --restore my_code.md --restore-path restored/
```

---

## Library Usage

Add to your `Cargo.toml`:

```toml
src2md = "0.1"
```

### Generate Markdown

```rust
use src2md::{Config, run_src2md};
use std::path::PathBuf;
use std::collections::HashSet;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = Config {
        output_path: PathBuf::from("out.md"),
        ignore_file: None,
        specific_paths: HashSet::new(),
        project_root: std::env::current_dir()?,
        restore_input: None,
        restore_path: None,
        verbosity: 0,
        fail_fast: true,
        extensions: HashSet::new(), // empty = include all
    };

    run_src2md(config).await
}
```

### Filter by Extension

```rust
use src2md::{Config, run_src2md};
use std::path::PathBuf;
use std::collections::HashSet;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Only include .rs and .toml files
    let mut extensions = HashSet::new();
    extensions.insert("rs".to_string());
    extensions.insert("toml".to_string());

    let config = Config {
        output_path: PathBuf::from("rust_only.md"),
        ignore_file: None,
        specific_paths: HashSet::new(),
        project_root: std::env::current_dir()?,
        restore_input: None,
        restore_path: None,
        verbosity: 0,
        fail_fast: true,
        extensions,
    };

    run_src2md(config).await
}
```

### Restore Files from Markdown

```rust
use src2md::extract_from_markdown;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    extract_from_markdown(&PathBuf::from("out.md"), Some(&PathBuf::from("restored/"))).await
}
```

---

## Changelog

### v0.1.5
- **New:** `--ext` flag to filter files by extension
- **New:** Automatic exclusion of hidden files and directories
- **New:** Automatic exclusion of lock files (`Cargo.lock`, `package-lock.json`, etc.)
- **New:** Automatic exclusion of previous src2md output files
- **New:** `--verbose` and `--fail-fast` flags
- **Changed:** Renamed `--extract` to `--restore` for clarity
- **Changed:** Renamed `-i, --ignore` to `--ignore-file` for clarity

### v0.1.1
- CLI and library mode
- Safe Markdown code fencing
- Extract mode with `--extract` and `--extract-path`

### v0.1.0
- Initial release
- Memory-mapped (zero-copy) file reading

---

## Contributors

- [Matias Hiltunen](https://github.com/MatiasHiltunen) – Author  
- You? [Submit a pull request](https://github.com/MatiasHiltunen/src2md/pulls)!

---

## License

MIT © [Matias Hiltunen](https://github.com/MatiasHiltunen)

# src2md

[![crates.io](https://img.shields.io/crates/v/src2md.svg)](https://crates.io/crates/src2md)
[![CI](https://github.com/MatiasHiltunen/src2md/actions/workflows/ci.yml/badge.svg)](https://github.com/MatiasHiltunen/src2md/actions/workflows/ci.yml)
[![license](https://img.shields.io/crates/l/src2md.svg)](https://github.com/MatiasHiltunen/src2md/blob/main/LICENSE)

A CLI tool that bundles source files into a single Markdown document. You can also restore the original files from the Markdown output.

Useful for sharing code with LLMs, creating documentation snapshots, or archiving projects in a readable format.

> **Note:** This project was developed with AI assistance. The codebase is tested but may have rough edges.

## Installation

```bash
cargo install src2md
```

Or download a binary from [Releases](https://github.com/MatiasHiltunen/src2md/releases).

## Quick Start

```bash
# Bundle current directory into Markdown
src2md -o project.md

# Only include certain file types
src2md --ext rs,toml -o rust_code.md

# Restore files from a bundle
src2md --restore project.md --restore-path ./restored/
```

## What It Does

- Walks directories and collects text files
- Wraps each file in a fenced code block with syntax highlighting
- Handles nested code blocks safely (uses extended backtick fences)
- Skips binary files (lists them without content)
- Can restore the original files from the Markdown output

## What It Excludes (by default)

- Hidden files and directories (`.git`, `.env`, etc.)
- Lock files (`Cargo.lock`, `package-lock.json`, `yarn.lock`, etc.)
- Its own previous output files

## CLI Options

```
src2md [OPTIONS] [PATHS]...

Arguments:
  [PATHS]...              Files or directories to include

Options:
  -o, --output <FILE>     Output file (default: {project}_{timestamp}.md)
  --ignore-file <FILE>    Custom ignore file (like .gitignore)
  -e, --ext <EXT>         Filter by extensions (comma-separated: rs,ts,js)
  -v, --verbose           Increase verbosity (-v, -vv, -vvv)
  --restore <FILE>        Restore files from a Markdown bundle
  --restore-path <DIR>    Where to restore files
  --fail-fast             Stop on first error
  -h, --help              Print help
```

## Library Usage

```rust
use src2md::{Config, run_src2md};
use std::collections::HashSet;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = Config {
        output_path: PathBuf::from("output.md"),
        project_root: std::env::current_dir()?,
        ignore_file: None,
        specific_paths: HashSet::new(),
        extensions: HashSet::new(),
        restore_input: None,
        restore_path: None,
        verbosity: 0,
        fail_fast: false,
    };
    
    run_src2md(config).await
}
```

See [docs.rs](https://docs.rs/src2md) for full API documentation.

## License

MIT

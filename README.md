
# src2md

A CLI tool and Rust library that collects source code and text files into a single Markdown (`.md`) document with syntax highlighting.

## Features

- Recursively scans directories to find files.
- Supports custom ignore files (`.src2md.ignore`, `.gitignore`, or user-defined).
- Option to include specific files or directories only.
- Wraps code in Markdown code blocks with language tags.
- Lists binary files by their paths without including content.
- Zero-copy file reading using memory-mapped files (`mmap`).
- Auto-generates output file name if not provided: `{project_folder}_content_{timestamp}.md`.

---

## Installation

### Prerequisites

- [Rust and Cargo](https://www.rust-lang.org/tools/install)

### Install via Cargo

```bash
cargo install --git https://github.com/MatiasHiltunen/src2md.git

Build from Source

git clone https://github.com/MatiasHiltunen/src2md.git
cd src2md
cargo build --release

Creates an executable at target/release/src2md.


---

Usage (CLI)

./target/release/src2md [OPTIONS] [PATHS]...

Options

Examples

# Default usage
src2md

# Specify output path
src2md -o docs/all_code.md

# Use custom ignore file
src2md -i custom.ignore

# Include specific files/directories
src2md src/main.rs src/lib.rs

# Combine options
src2md -o out.md -i .gitignore src/ tests/


---

Usage (Library)

Add to your Cargo.toml:

src2md = { git = "https://github.com/MatiasHiltunen/src2md" }

And use it:

use src2md::{Config, run_src2md};
use std::collections::HashSet;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = Config {
        output_path: PathBuf::from("out.md"),
        ignore_file: None,
        specific_paths: HashSet::new(),
        project_root: std::env::current_dir()?,
    };

    run_src2md(config).await
}


---

Testing

cargo test

Includes integration tests for file scanning and markdown generation.


---

License

MIT © Matias Hiltunen

Contributions welcome!

---


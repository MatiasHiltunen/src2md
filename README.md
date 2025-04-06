# src2md

[![crates.io](https://img.shields.io/crates/v/src2md.svg)](https://crates.io/crates/src2md)
[![docs.rs](https://docs.rs/src2md/badge.svg)](https://docs.rs/src2md)
[![license](https://img.shields.io/crates/l/src2md.svg)](https://github.com/MatiasHiltunen/src2md/blob/main/LICENSE)

Turn source/text files into a single Markdown document — or extract them back. Built with Rust **2024 edition**.

> **Warning**  
> This project is in early development and may change rapidly.  
> AI (ChatGPT) has been extensively used in the design and implementation of this codebase.  
> Review carefully before using in production or contributing.

---

## Features

- Recursively scans directories to find files.
- Supports `.src2md.ignore`, `.gitignore`, or a custom ignore file.
- Option to include specific files or directories.
- Wraps content in Markdown code blocks with syntax highlighting.
- Uses dynamic backtick fencing to safely include Markdown and code.
- Lists binary files by their paths (content omitted).
- **Zero-copy** file reading using memory-mapped files.
- Extracts files back from a generated Markdown file (with `--extract`).
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

| Flag                  | Description                                                        |
|-----------------------|--------------------------------------------------------------------|
| `-o, --output FILE`    | Output Markdown file (default: `{project}_content_{timestamp}.md`) |
| `-i, --ignore FILE`    | Ignore file path (`.src2md.ignore` or `.gitignore` by default)     |
| `[PATHS]`              | Files or directories to include                                    |
| `--extract FILE.md`    | Extracts original files from a `.md` file                          |
| `--extract-path DIR`   | Target folder to extract files into                                |

### Examples

```bash
# Default: all files in current dir → Markdown
src2md

# Specify output path
src2md -o docs/code.md

# Use custom ignore file
src2md -i .customignore

# Include only certain files
src2md src/lib.rs src/main.rs

# Extract files back from Markdown
src2md --extract my_code.md --extract-path restored/
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
        extract_input: None,
        extract_path: None,
    };

    run_src2md(config).await
}
```

### Extract Files from Markdown

```rust
use src2md::extract_from_markdown;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    extract_from_markdown(&PathBuf::from("out.md"), Some(&PathBuf::from("restored/"))).await
}
```

---

## Cargo Features

_Planned (not yet active):_

- `highlight`: syntax highlighting with `syntect`
- `serde`: config serialization
- `cli-only`: trim library for tiny builds

---

## Changelog

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

# src2md

[![crates.io](https://img.shields.io/crates/v/src2md.svg)](https://crates.io/crates/src2md)
[![CI](https://github.com/MatiasHiltunen/src2md/actions/workflows/ci.yml/badge.svg)](https://github.com/MatiasHiltunen/src2md/actions/workflows/ci.yml)
[![license](https://img.shields.io/crates/l/src2md.svg)](https://github.com/MatiasHiltunen/src2md/blob/main/LICENSE)

A CLI tool that bundles text files into a single Markdown document. You can also restore the original files from the Markdown output.

Useful for sharing code with LLMs, creating documentation snapshots, or archiving projects in a readable format.

> **Note:** This project was developed with AI assistance. The codebase is tested and verified by human.

## Installation

```bash
cargo install src2md
```

Or download a binary from [Releases](https://github.com/MatiasHiltunen/src2md/releases).

<details>
<summary><strong>macOS: Allow unsigned binary</strong></summary>

Downloaded binaries are blocked by Gatekeeper. To allow:

```bash
# Remove quarantine attribute after extracting
xattr -d com.apple.quarantine src2md

# Or: Right-click → Open → Open anyway
```

</details>

## Quick Start

```bash
# Bundle current directory into Markdown
src2md -o project.md

# Only include certain file types
src2md --ext rs,toml -o rust_code.md

# Bundle a remote git repository
src2md --git https://github.com/user/repo -o repo.md

# Generate mdbook format output
src2md --mdbook ./book/src

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

## Usage Examples

### Bundle a Local Project

```bash
# Bundle everything in current directory
src2md -o output.md

# Bundle specific directories or files
src2md src/ tests/ README.md -o output.md

# Only include Rust and TOML files
src2md --ext rs,toml -o rust_only.md

# Use a custom ignore file
src2md --ignore-file .myignore -o output.md
```

### Clone and Bundle a Git Repository

The `--git` flag clones a repository to a temporary directory, bundles it, and cleans up automatically:

```bash
# Clone and bundle a public repository
src2md --git https://github.com/rust-lang/rust-by-example -o rust_examples.md

# Specify a branch
src2md --git https://github.com/user/repo --branch develop -o output.md

# Combine with extension filter
src2md --git https://github.com/user/repo --ext rs,md -o filtered.md
```

The output filename defaults to `{repo_name}_content_{timestamp}.md` if not specified.

### Generate mdbook Format

The `--mdbook` flag generates output compatible with [mdbook](https://rust-lang.github.io/mdBook/):

```bash
# Generate mdbook source files
src2md --mdbook ./book/src

# Then build with mdbook
mdbook build book/
```

This creates:
- `SUMMARY.md` with chapter structure based on directory layout
- One `.md` file per folder, with each file as a section
- Nested folders become nested chapters

Example structure:
```
book/src/
  SUMMARY.md
  introduction.md    # root files
  src.md             # files from src/
  src/
    utils.md         # files from src/utils/
```

### Restore Files from Markdown

The `--restore` flag extracts files from a src2md-generated Markdown back to the filesystem:

```bash
# Restore to a specific directory
src2md --restore project.md --restore-path ./restored/

# Restore to current directory (recreates original structure)
src2md --restore project.md
```

This recreates the original directory structure and file contents. Useful for:
- Recovering code shared in a Markdown document
- Unpacking code snippets from LLM conversations
- Reverting to a previous snapshot

## CLI Reference

```
src2md [OPTIONS] [PATHS]...

Arguments:
  [PATHS]...                Files or directories to include

Options:
  -o, --output <FILE>       Output file (default: {project}_{timestamp}.md)
  --ignore-file <FILE>      Custom ignore file (like .gitignore)
  -e, --ext <EXT>           Filter by extensions (comma-separated: rs,ts,js)
  -v, --verbose             Increase verbosity (-v, -vv, -vvv)
  --git <URL>               Clone and bundle a git repository
  -b, --branch <BRANCH>     Git branch to checkout (requires --git)
  --mdbook <DIR>            Generate mdbook format to directory
  --restore <FILE>          Restore files from a Markdown bundle
  --restore-path <DIR>      Target directory for restore (default: current dir)
  --fail-fast               Stop on first error
  -h, --help                Print help
  -V, --version             Print version
```

## Library Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
src2md = "0.1"
```

### Feature Flags

All features are enabled by default:

| Feature   | Description                                    |
|-----------|------------------------------------------------|
| `restore` | Enables `--restore` flag and `extract_from_markdown` API |
| `git`     | Enables `--git <URL>` to clone and process repositories |
| `mdbook`  | Enables `--mdbook <DIR>` for mdbook format output |

To use only the core bundling functionality:

```toml
src2md = { version = "0.1", default-features = false }
```

### Example

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
        #[cfg(feature = "restore")]
        restore_input: None,
        #[cfg(feature = "restore")]
        restore_path: None,
        verbosity: 0,
        fail_fast: false,
        #[cfg(feature = "git")]
        git_url: None,
        #[cfg(feature = "git")]
        git_branch: None,
        #[cfg(feature = "mdbook")]
        mdbook_output: None,
    };
    
    run_src2md(config).await
}
```

See [docs.rs](https://docs.rs/src2md) for full API documentation.

## License

MIT

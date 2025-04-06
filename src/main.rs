//! src2md - Collects code and text files into a single Markdown file
//!
//! ## Features
//! - Recursively scans directories to find files.
//! - Supports custom ignore files, defaulting to .src2md.ignore or .gitignore.
//! - Option to include specific files or directories.
//! - Wraps code in Markdown code blocks with appropriate language tags for syntax highlighting.
//! - Lists binary files by their paths without including content.
//! - Zero-Copy File Reading using memory-mapped files for efficient file reading.
//!
//! ## Installation
//! ### Prerequisites
//! Ensure you have Rust and Cargo installed.
//! To install src2md with cargo, run:
//!     cargo install --git https://github.com/MatiasHiltunen/src2md.git
//!
//! ### Build from Source
//! Clone the repository and build the project:
//!     git clone https://github.com/yourusername/src2md.git
//!     cd src2md
//!     cargo build --release
//! This will create an executable in target/release/src2md.
//!
//! ## Usage
//!     ./target/release/src2md [OPTIONS] [PATHS]...
//!
//! ### Options
//! -o, --output : Sets output file (default: all_the_code.md)
//! -i, --ignore : Sets custom ignore file
//! [PATHS]      : Specific paths to include
//!
//! ### Examples
//!     ./target/release/src2md
//!     ./target/release/src2md -o docs/code.md
//!     ./target/release/src2md -i custom.ignore src/ lib.rs

mod cli;
mod filewalker;
mod utils;
mod writer;

use crate::cli::parse_args;
use crate::filewalker::collect_files;
use crate::writer::MarkdownWriter;
use anyhow::Result;
use tokio::fs::File;
use tokio::io::BufWriter;

#[tokio::main]
async fn main() -> Result<()> {
    let config = parse_args()?;

    let file = File::create(&config.output_path).await?;
    let buf_writer = BufWriter::new(file);
    let mut md_writer = MarkdownWriter::new(buf_writer);

    let entries = collect_files(
        &config.project_root,
        config.ignore_file.as_ref(),
        &config.specific_paths,
    )?;

    for entry in entries {
        md_writer.write_entry(&entry, &config.project_root).await?;
    }

    Ok(())
}

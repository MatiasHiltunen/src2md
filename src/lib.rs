//! # src2md Library
//!
//! This crate can be used to:
//!
//! - Collect all source/text files from a project and compile them into a Markdown file
//! - Extract original source files back from a generated Markdown file
//!
//! ## Usage
//!
//! ### To generate a Markdown file:
//!
//! ```rust
//! use src2md::{Config, run_src2md};
//! use std::collections::HashSet;
//! use std::path::PathBuf;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let config = Config {
//!         output_path: PathBuf::from("output.md"),
//!         ignore_file: None,
//!         specific_paths: HashSet::new(),
//!         project_root: std::env::current_dir()?,
//!         extract_input: None,
//!         extract_path: None,
//!     };
//!
//!     run_src2md(config).await
//! }
//! ```
//!
//! ### To extract files from a Markdown file:
//!
//! ```rust
//! use src2md::extract_from_markdown;
//! use std::path::PathBuf;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     extract_from_markdown(&PathBuf::from("output.md"), Some(&PathBuf::from("restored/"))).await
//! }
//! ```

pub mod cli;
pub mod extractor;
pub mod filewalker;
pub mod utils;
pub mod writer;

pub use cli::Config;
pub use extractor::extract_from_markdown;
pub use filewalker::collect_files;
pub use writer::MarkdownWriter;

use anyhow::Result;
use tokio::fs::File;
use tokio::io::BufWriter;

/// Generate a Markdown file from source/text files
pub async fn run_src2md(config: Config) -> Result<()> {
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

    md_writer.flush().await?;
    Ok(())
}

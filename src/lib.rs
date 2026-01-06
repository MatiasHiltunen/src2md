//! # src2md Library
//!
//! This crate can be used to:
//!
//! - Collect all source/text files from a project and compile them into a Markdown file
//! - Restore original source files back from a generated Markdown file
//! - Clone and process git repositories (with the `git` feature)
//!
//! ## Features
//!
//! - `git` - Enables git repository cloning support via `--git <url>`
//!
//! ## Default Exclusions
//!
//! The following are always excluded by default:
//! - Hidden files and directories (starting with `.`)
//! - Lock files (package-lock.json, yarn.lock, Cargo.lock, etc.)
//! - Previous src2md output files
//!
//! ## Usage
//!
//! ### To generate a Markdown file:
//!
//! ```rust,no_run
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
//!         restore_input: None,
//!         restore_path: None,
//!         verbosity: 0,
//!         fail_fast: true,
//!         extensions: HashSet::new(), // empty = include all
//!         #[cfg(feature = "git")]
//!         git_url: None,
//!         #[cfg(feature = "git")]
//!         git_branch: None,
//!     };
//!
//!     run_src2md(config).await
//! }
//! ```
//!
//! ### To restore files from a Markdown file:
//!
//! ```rust,no_run
//! use src2md::extract_from_markdown;
//! use std::path::PathBuf;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     extract_from_markdown(
//!         &PathBuf::from("generated.md"),
//!         Some(&PathBuf::from("restored/")),
//!     ).await
//! }
//! ```

pub mod cli;
pub mod extractor;
pub mod filewalker;
pub mod utils;
pub mod writer;

#[cfg(feature = "git")]
pub mod git;

pub use cli::Config;
pub use extractor::extract_from_markdown;
pub use filewalker::collect_files;
pub use writer::{MarkdownWriter, OUTPUT_MAGIC_BYTES, OUTPUT_MAGIC_HEADER};

#[cfg(feature = "git")]
pub use git::{ClonedRepo, clone_repository, repo_name_from_url};

use anyhow::Result;
use log::error;
use tokio::fs::File;
use tokio::io::BufWriter;

/// Generate a Markdown file from source/text files
///
/// If `fail_fast` is true in the config, stops on first error.
/// Otherwise, logs errors and continues processing remaining files.
///
/// # Output File Handling
///
/// The output file and any previous src2md outputs are automatically excluded
/// from collection to prevent:
/// - Race conditions (writing while reading the same file)
/// - Self-inclusion (including previous outputs in new outputs)
///
/// # Default Exclusions
///
/// Hidden files, lock files, and previous src2md outputs are always excluded.
/// Use the `extensions` field to filter by file type.
pub async fn run_src2md(config: Config) -> Result<()> {
    let file = File::create(&config.output_path).await?;
    let buf_writer = BufWriter::new(file);
    let mut md_writer = MarkdownWriter::new(buf_writer);

    let entries = collect_files(
        &config.project_root,
        config.ignore_file.as_ref(),
        &config.specific_paths,
        Some(&config.output_path),
        &config.extensions,
    )?;

    for entry in entries {
        if let Err(e) = md_writer.write_entry(&entry, &config.project_root).await {
            if config.fail_fast {
                return Err(e);
            }
            error!("Failed to write {}: {e}", entry.path().display());
        }
    }

    md_writer.flush().await?;
    Ok(())
}

/// Generate a Markdown file from a specific directory path.
///
/// This is a convenience function that creates a Config and runs src2md.
/// Useful when you have a path (e.g., from a cloned git repo) and want to
/// process it without constructing a full Config.
pub async fn run_src2md_on_path(
    project_root: std::path::PathBuf,
    output_path: std::path::PathBuf,
    ignore_file: Option<std::path::PathBuf>,
    extensions: &std::collections::HashSet<String>,
    fail_fast: bool,
) -> Result<()> {
    let file = File::create(&output_path).await?;
    let buf_writer = BufWriter::new(file);
    let mut md_writer = MarkdownWriter::new(buf_writer);

    let entries = collect_files(
        &project_root,
        ignore_file.as_ref(),
        &std::collections::HashSet::new(),
        Some(&output_path),
        extensions,
    )?;

    for entry in entries {
        if let Err(e) = md_writer.write_entry(&entry, &project_root).await {
            if fail_fast {
                return Err(e);
            }
            error!("Failed to write {}: {e}", entry.path().display());
        }
    }

    md_writer.flush().await?;
    Ok(())
}

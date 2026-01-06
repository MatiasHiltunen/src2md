use anyhow::Result;
use log::{LevelFilter, error, info};
use src2md::cli::parse_args;
#[cfg(feature = "restore")]
use src2md::extractor::extract_from_markdown;
use src2md::filewalker::collect_files;
#[cfg(feature = "mdbook")]
use src2md::mdbook::generate_mdbook;
use src2md::writer::MarkdownWriter;
use tokio::fs::File;
use tokio::io::BufWriter;

fn init_logger(verbosity: u8) {
    let level = match verbosity {
        0 => LevelFilter::Warn,
        1 => LevelFilter::Info,
        2 => LevelFilter::Debug,
        _ => LevelFilter::Trace,
    };

    env_logger::Builder::new()
        .filter_level(level)
        .format_target(false)
        .format_timestamp_secs()
        .init();
}

#[tokio::main]
async fn main() -> Result<()> {
    let config = parse_args()?;
    init_logger(config.verbosity);

    // Handle restore mode (requires feature)
    #[cfg(feature = "restore")]
    if let Some(input) = &config.restore_input {
        info!("Restoring files from: {}", input.display());
        extract_from_markdown(input, config.restore_path.as_ref()).await?;
        info!("Restore complete");
        return Ok(());
    }

    // Handle git mode (requires feature)
    #[cfg(feature = "git")]
    if let Some(ref git_url) = config.git_url {
        return run_git_mode(&config, git_url).await;
    }

    // Handle mdbook mode (requires feature)
    #[cfg(feature = "mdbook")]
    if let Some(ref output_dir) = config.mdbook_output {
        return run_mdbook_mode(&config, output_dir).await;
    }

    // Standard mode: process local directory
    run_local_mode(&config).await
}

/// Process a local directory and generate markdown output.
async fn run_local_mode(config: &src2md::Config) -> Result<()> {
    info!("Output file: {}", config.output_path.display());

    if !config.extensions.is_empty() {
        info!("Filtering by extensions: {:?}", config.extensions);
    }

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

    info!("Processing {} files", entries.len());

    for entry in entries {
        if let Err(e) = md_writer.write_entry(&entry, &config.project_root).await {
            error!("Failed to write {}: {e}", entry.path().display());
            if config.fail_fast {
                return Err(e);
            }
        }
    }

    md_writer.flush().await?;
    info!("Done: {}", config.output_path.display());

    Ok(())
}

/// Clone a git repository and generate markdown from it.
#[cfg(feature = "git")]
async fn run_git_mode(config: &src2md::Config, git_url: &str) -> Result<()> {
    use src2md::git::clone_repository;

    info!("Cloning repository: {}", git_url);

    // Clone the repository
    let cloned = clone_repository(git_url, config.git_branch.as_deref())?;
    let project_root = cloned.path().clone();

    info!("Output file: {}", config.output_path.display());

    if !config.extensions.is_empty() {
        info!("Filtering by extensions: {:?}", config.extensions);
    }

    let file = File::create(&config.output_path).await?;
    let buf_writer = BufWriter::new(file);
    let mut md_writer = MarkdownWriter::new(buf_writer);

    // Look for .gitignore in the cloned repo to use as ignore file
    let ignore_file = config.ignore_file.clone().or_else(|| {
        let gitignore = project_root.join(".gitignore");
        if gitignore.exists() {
            Some(gitignore)
        } else {
            None
        }
    });

    let entries = collect_files(
        &project_root,
        ignore_file.as_ref(),
        &config.specific_paths,
        Some(&config.output_path),
        &config.extensions,
    )?;

    info!("Processing {} files from cloned repository", entries.len());

    for entry in entries {
        if let Err(e) = md_writer.write_entry(&entry, &project_root).await {
            error!("Failed to write {}: {e}", entry.path().display());
            if config.fail_fast {
                return Err(e);
            }
        }
    }

    md_writer.flush().await?;
    info!("Done: {}", config.output_path.display());

    // The cloned repo is automatically cleaned up when `cloned` is dropped
    info!("Cleaned up temporary clone");

    Ok(())
}

/// Generate mdbook format output from a local directory.
#[cfg(feature = "mdbook")]
async fn run_mdbook_mode(config: &src2md::Config, output_dir: &std::path::Path) -> Result<()> {
    info!("Output directory: {}", output_dir.display());

    if !config.extensions.is_empty() {
        info!("Filtering by extensions: {:?}", config.extensions);
    }

    let entries = collect_files(
        &config.project_root,
        config.ignore_file.as_ref(),
        &config.specific_paths,
        None, // No single output file to exclude
        &config.extensions,
    )?;

    info!("Processing {} files into mdbook format", entries.len());

    generate_mdbook(&entries, &config.project_root, output_dir).await?;

    info!("Done: {}", output_dir.display());
    Ok(())
}

pub mod cli;
pub mod filewalker;
pub mod utils;
pub mod writer;

pub use cli::Config;
pub use filewalker::collect_files;
pub use writer::MarkdownWriter;

use anyhow::Result;
use tokio::fs::File;
use tokio::io::BufWriter;

/// Public entry point for using src2md as a library
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

    Ok(())
}

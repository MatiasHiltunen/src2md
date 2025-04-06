mod cli;
mod extractor;
mod filewalker;
mod utils;
mod writer;

use crate::cli::parse_args;
use crate::extractor::extract_from_markdown;
use crate::filewalker::collect_files;
use crate::writer::MarkdownWriter;
use anyhow::Result;
use tokio::fs::File;
use tokio::io::BufWriter;

#[tokio::main]
async fn main() -> Result<()> {
    let config = parse_args()?;

    if let Some(input) = &config.extract_input {
        extract_from_markdown(input, config.extract_path.as_ref()).await?;
    } else {
        let file = File::create(&config.output_path).await?;
        let buf_writer = BufWriter::new(file);
        let mut md_writer = MarkdownWriter::new(buf_writer);

        let entries = collect_files(
            &config.project_root,
            config.ignore_file.as_ref(),
            &config.specific_paths,
        )?;

        for entry in entries {
            if let Err(e) = md_writer.write_entry(&entry, &config.project_root).await {
                eprintln!("Failed to write {}: {e}", entry.path().display());
            }
        }

        md_writer.flush().await?;
    }

    Ok(())
}

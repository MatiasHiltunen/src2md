use crate::utils::get_language_tag;
use anyhow::{Context, Result};
use content_inspector::{ContentType, inspect};
use ignore::DirEntry;
use log::debug;
use memmap2::MmapOptions;
use std::fs::File as StdFile;
use std::path::Path;
use std::str;
use tokio::fs::File;
use tokio::io::{AsyncWriteExt, BufWriter};

pub struct MarkdownWriter<W: AsyncWriteExt + Unpin> {
    writer: BufWriter<W>,
}

impl MarkdownWriter<tokio::fs::File> {
    pub fn new(writer: BufWriter<File>) -> Self {
        Self { writer }
    }

    pub async fn write_entry(&mut self, entry: &DirEntry, project_root: &Path) -> Result<()> {
        let path = entry.path();
        let rel_path = path.strip_prefix(project_root).unwrap_or(path);

        debug!("Writing file: {}", rel_path.display());

        // Write header
        self.writer
            .write_all(format!("## {}\n\n", rel_path.display()).as_bytes())
            .await
            .with_context(|| format!("Failed to write heading for {}", rel_path.display()))?;

        // Open and map file
        let file = StdFile::open(path)
            .with_context(|| format!("Failed to open file: {}", path.display()))?;

        let mmap = unsafe {
            MmapOptions::new()
                .map(&file)
                .with_context(|| format!("Failed to mmap file: {}", path.display()))?
        };

        if mmap.len() == 0 {
            debug!(
                "WARNING: File '{}' was mmap'd but is empty!",
                path.display()
            );
        }

        // Preview the start of the file
        let preview_len = std::cmp::min(100, mmap.len());
        if let Ok(preview) = std::str::from_utf8(&mmap[..preview_len]) {
            debug!("Preview: {:?}", preview);
        }

        // Detect binary vs text
        let sample_size = std::cmp::min(8192, mmap.len());
        let content_type = inspect(&mmap[..sample_size]);

        if content_type == ContentType::BINARY {
            self.writer
                .write_all(b"(binary file omitted)\n\n")
                .await
                .with_context(|| {
                    format!("Failed to write binary marker for {}", rel_path.display())
                })?;
        } else {
            let lang = get_language_tag(path);
            self.writer
                .write_all(format!("```{}\n", lang).as_bytes())
                .await
                .with_context(|| {
                    format!(
                        "Failed to write opening code fence for {}",
                        rel_path.display()
                    )
                })?;

            // Attempt to write from memory-mapped data
            if let Ok(text) = str::from_utf8(&mmap) {
                self.writer
                    .write_all(text.as_bytes())
                    .await
                    .with_context(|| {
                        format!("Failed to write UTF-8 content from {}", rel_path.display())
                    })?;
            } else {
                // Fallback to read_to_string
                debug!(
                    "Invalid UTF-8 in {}, falling back to read_to_string",
                    rel_path.display()
                );
                let content = std::fs::read_to_string(path)
                    .with_context(|| format!("Fallback read failed for {}", path.display()))?;
                self.writer
                    .write_all(content.as_bytes())
                    .await
                    .with_context(|| {
                        format!("Failed to write fallback string for {}", rel_path.display())
                    })?;
            }

            self.writer.write_all(b"\n```\n\n").await.with_context(|| {
                format!(
                    "Failed to write closing code fence for {}",
                    rel_path.display()
                )
            })?;
        }

        self.writer.flush().await?;
        debug!("Finished: {}\n", rel_path.display());
        Ok(())
    }

    pub async fn flush(&mut self) -> Result<()> {
        self.writer.flush().await.context("Failed to flush output")
    }
}


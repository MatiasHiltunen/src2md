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

        self.writer
            .write_all(format!("## {}\n\n", rel_path.display()).as_bytes())
            .await
            .with_context(|| format!("Failed to write heading for {}", rel_path.display()))?;

        let file = StdFile::open(path)
            .with_context(|| format!("Failed to open file: {}", path.display()))?;

        let mmap = unsafe {
            MmapOptions::new()
                .map(&file)
                .with_context(|| format!("Failed to mmap file: {}", path.display()))?
        };

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

            // Fully own content string safely
            let content: String = match str::from_utf8(&mmap) {
                Ok(s) => s.to_string(),
                Err(_) => std::fs::read_to_string(path)
                    .with_context(|| format!("Fallback read failed for {}", path.display()))?,
            };
            let text = content.as_str();

            // Determine how many backticks exist in content
            let max_backtick_run = text
                .lines()
                .filter_map(|line| {
                    let trimmed = line.trim_start();
                    if trimmed.starts_with('`') {
                        Some(trimmed.chars().take_while(|&c| c == '`').count())
                    } else {
                        None
                    }
                })
                .max()
                .unwrap_or(2);

            let fence = "`".repeat(max_backtick_run + 1);

            self.writer
                .write_all(format!("{}{}\n", fence, lang).as_bytes())
                .await
                .with_context(|| {
                    format!("Failed to write opening fence for {}", rel_path.display())
                })?;

            self.writer
                .write_all(text.as_bytes())
                .await
                .with_context(|| format!("Failed to write content for {}", rel_path.display()))?;

            self.writer
                .write_all(format!("\n{}\n\n", fence).as_bytes())
                .await
                .with_context(|| {
                    format!("Failed to write closing fence for {}", rel_path.display())
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

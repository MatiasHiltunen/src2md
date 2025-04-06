use crate::utils::get_language_tag;
use anyhow::Result;
use content_inspector::{inspect, ContentType};
use ignore::DirEntry;
use memmap2::Mmap;
use std::fs::File as StdFile;
use std::path::Path;
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
        let rel_path = path.strip_prefix(project_root)?;
        let file = StdFile::open(path)?;
        let mmap = unsafe { Mmap::map(&file)? };

        let sample_size = std::cmp::min(8192, mmap.len());
        let content_type = inspect(&mmap[..sample_size]);

        self.writer
            .write_all(format!("## {}\n\n", rel_path.display()).as_bytes())
            .await?;

        if content_type == ContentType::BINARY {
            self.writer.write_all(b"(binary file omitted)\n\n").await?;
        } else {
            let lang = get_language_tag(path);
            self.writer
                .write_all(format!("```{}\n", lang).as_bytes())
                .await?;
            self.writer.write_all(&mmap).await?;
            self.writer.write_all(b"\n```\n\n").await?;
        }

        Ok(())
    }
}

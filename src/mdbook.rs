//! mdbook output format generation.
//!
//! This module generates mdbook-compatible output from collected files:
//! - SUMMARY.md with chapter structure
//! - One .md file per folder containing all files as sections
//! - Nested folders become nested chapters

use crate::utils::get_language_tag;
use anyhow::{Context, Result};
use content_inspector::{ContentType, inspect};
use ignore::DirEntry;
use log::{debug, info};
use memmap2::MmapOptions;
use std::collections::BTreeMap;
use std::fs::File as StdFile;
use std::future::Future;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::str;
use tokio::fs;

/// Represents a chapter in the mdbook structure.
/// Each chapter corresponds to a directory and contains files as sections.
#[derive(Debug, Default)]
struct Chapter {
    /// Files directly in this directory (relative paths within project)
    files: Vec<PathBuf>,
    /// Nested chapters (subdirectories)
    children: BTreeMap<String, Chapter>,
}

impl Chapter {
    fn new() -> Self {
        Self::default()
    }

    /// Inserts a file into the appropriate chapter based on its path.
    fn insert(&mut self, rel_path: &Path) {
        let components: Vec<_> = rel_path.components().collect();

        if components.len() == 1 {
            // File at this level
            self.files.push(rel_path.to_path_buf());
        } else {
            // File in a subdirectory
            let dir_name = components[0].as_os_str().to_string_lossy().to_string();
            let remaining: PathBuf = components[1..].iter().collect();

            self.children
                .entry(dir_name)
                .or_default()
                .insert(&remaining);
        }
    }

    /// Returns true if this chapter has any content (files or children with content).
    fn has_content(&self) -> bool {
        !self.files.is_empty() || self.children.values().any(|c| c.has_content())
    }
}

/// Generates mdbook output from collected files.
pub struct MdbookWriter {
    /// Root chapter containing the file structure
    root: Chapter,
    /// Project root for reading files
    project_root: PathBuf,
    /// Output directory for mdbook src/
    output_dir: PathBuf,
}

impl MdbookWriter {
    /// Creates a new MdbookWriter.
    pub fn new(project_root: PathBuf, output_dir: PathBuf) -> Self {
        Self {
            root: Chapter::new(),
            project_root,
            output_dir,
        }
    }

    /// Adds files to the chapter structure.
    pub fn add_files(&mut self, entries: &[DirEntry]) {
        for entry in entries {
            let path = entry.path();
            if let Ok(rel_path) = path.strip_prefix(&self.project_root) {
                self.root.insert(rel_path);
            }
        }
    }

    /// Writes the complete mdbook structure to the output directory.
    pub async fn write(&self) -> Result<()> {
        // Create output directory
        fs::create_dir_all(&self.output_dir)
            .await
            .with_context(|| {
                format!("Failed to create output dir: {}", self.output_dir.display())
            })?;

        // Generate and write SUMMARY.md
        let summary = self.generate_summary();
        let summary_path = self.output_dir.join("SUMMARY.md");
        fs::write(&summary_path, summary)
            .await
            .with_context(|| "Failed to write SUMMARY.md")?;
        info!("Wrote: {}", summary_path.display());

        // Write chapter files
        self.write_chapters(&self.root, &PathBuf::new(), &PathBuf::new(), 0)
            .await?;

        Ok(())
    }

    /// Generates the SUMMARY.md content.
    fn generate_summary(&self) -> String {
        let mut summary = String::from("# Summary\n\n");

        // Root files go into "Introduction"
        if !self.root.files.is_empty() {
            summary.push_str("- [Introduction](./introduction.md)\n");
        }

        // Add chapters for each top-level directory
        self.append_summary_entries(&mut summary, &self.root.children, &PathBuf::new(), 0);

        summary
    }

    /// Recursively appends chapter entries to the summary.
    fn append_summary_entries(
        &self,
        summary: &mut String,
        chapters: &BTreeMap<String, Chapter>,
        parent_path: &Path,
        depth: usize,
    ) {
        let indent = "  ".repeat(depth);

        for (name, chapter) in chapters {
            if !chapter.has_content() {
                continue;
            }

            let chapter_path = parent_path.join(name);
            let md_path = format!("./{}.md", chapter_path.display());

            summary.push_str(&format!("{}- [{}]({})\n", indent, name, md_path));

            // Recurse into children
            self.append_summary_entries(summary, &chapter.children, &chapter_path, depth + 1);
        }
    }

    /// Recursively writes chapter files.
    /// Uses Box::pin for async recursion.
    ///
    /// # Arguments
    /// * `chapter` - The chapter to write
    /// * `rel_path` - The relative path to this chapter from the output dir (for .md file naming)
    /// * `src_path` - The source path prefix for finding files in project_root
    /// * `depth` - Current depth (0 = root)
    fn write_chapters<'a>(
        &'a self,
        chapter: &'a Chapter,
        rel_path: &'a Path,
        src_path: &'a Path,
        depth: usize,
    ) -> Pin<Box<dyn Future<Output = Result<()>> + Send + 'a>> {
        Box::pin(async move {
            // Write introduction.md for root files
            if depth == 0 && !chapter.files.is_empty() {
                let intro_path = self.output_dir.join("introduction.md");
                let content = self
                    .generate_chapter_content(&chapter.files, src_path, "Introduction")
                    .await?;
                fs::write(&intro_path, content)
                    .await
                    .with_context(|| "Failed to write introduction.md")?;
                info!("Wrote: {}", intro_path.display());
            }

            // Write chapter files for subdirectories
            for (name, child) in &chapter.children {
                let child_rel_path = rel_path.join(name);
                let child_src_path = src_path.join(name);

                // Write this chapter's file if it has files
                if !child.files.is_empty() {
                    let md_filename = format!("{}.md", child_rel_path.display());
                    let md_path = self.output_dir.join(&md_filename);

                    // Create parent directories if needed
                    if let Some(parent) = md_path.parent() {
                        fs::create_dir_all(parent).await.with_context(|| {
                            format!("Failed to create dir: {}", parent.display())
                        })?;
                    }

                    let content = self
                        .generate_chapter_content(&child.files, &child_src_path, name)
                        .await?;
                    fs::write(&md_path, content)
                        .await
                        .with_context(|| format!("Failed to write {}", md_path.display()))?;
                    info!("Wrote: {}", md_path.display());
                }

                // Recurse into children
                self.write_chapters(child, &child_rel_path, &child_src_path, depth + 1)
                    .await?;
            }

            Ok(())
        })
    }

    /// Generates the content for a chapter file.
    ///
    /// # Arguments
    /// * `files` - Filenames in this chapter (just the filename, not full path)
    /// * `src_path` - The source directory path prefix to find files
    /// * `title` - The chapter title
    async fn generate_chapter_content(
        &self,
        files: &[PathBuf],
        src_path: &Path,
        title: &str,
    ) -> Result<String> {
        let mut content = format!("# {}\n\n", title);

        for filename in files {
            // Construct full relative path: src_path + filename
            let rel_path = src_path.join(filename);
            let full_path = self.project_root.join(&rel_path);
            let file_content = self.read_file_content(&full_path, filename).await?;
            content.push_str(&file_content);
        }

        Ok(content)
    }

    /// Reads and formats a single file's content as a section.
    async fn read_file_content(&self, full_path: &Path, rel_path: &Path) -> Result<String> {
        let filename = rel_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("file");

        debug!("Processing: {}", rel_path.display());

        // Memory-map the file for efficient reading
        let file = StdFile::open(full_path)
            .with_context(|| format!("Failed to open file: {}", full_path.display()))?;

        let mmap = unsafe {
            MmapOptions::new()
                .map(&file)
                .with_context(|| format!("Failed to mmap file: {}", full_path.display()))?
        };

        // Check for binary content
        let sample_size = std::cmp::min(8192, mmap.len());
        let content_type = inspect(&mmap[..sample_size]);

        if content_type == ContentType::BINARY {
            return Ok(format!("## {}\n\n(binary file omitted)\n\n", filename));
        }

        // Get content as string
        let text = match str::from_utf8(&mmap) {
            Ok(s) => s.to_string(),
            Err(_) => std::fs::read_to_string(full_path)
                .with_context(|| format!("Fallback read failed for {}", full_path.display()))?,
        };

        // Get language tag for syntax highlighting
        let lang = get_language_tag(full_path);

        // Calculate fence length
        let fence = calculate_fence(&text);

        Ok(format!(
            "## {}\n\n{}{}\n{}\n{}\n\n",
            filename, fence, lang, text, fence
        ))
    }
}

/// Calculates the appropriate fence string for wrapping content.
///
/// Returns a fence with at least 3 backticks, or more if the content
/// contains backtick sequences that would interfere with parsing.
fn calculate_fence(content: &str) -> String {
    let max_backtick_run = content
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
        .unwrap_or(0);

    // Minimum fence length is 3, or one more than the max found in content
    let fence_len = max_backtick_run.max(2) + 1;
    "`".repeat(fence_len)
}

/// Generates mdbook output from collected files.
///
/// # Arguments
/// * `entries` - Collected file entries
/// * `project_root` - Root directory of the project being processed
/// * `output_dir` - Directory to write mdbook src/ contents
pub async fn generate_mdbook(
    entries: &[DirEntry],
    project_root: &Path,
    output_dir: &Path,
) -> Result<()> {
    let mut writer = MdbookWriter::new(project_root.to_path_buf(), output_dir.to_path_buf());
    writer.add_files(entries);
    writer.write().await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chapter_insert_single_file() {
        let mut chapter = Chapter::new();
        chapter.insert(Path::new("file.rs"));
        assert_eq!(chapter.files.len(), 1);
        assert!(chapter.children.is_empty());
    }

    #[test]
    fn test_chapter_insert_nested_file() {
        let mut chapter = Chapter::new();
        chapter.insert(Path::new("src/main.rs"));
        assert!(chapter.files.is_empty());
        assert!(chapter.children.contains_key("src"));
        assert_eq!(chapter.children["src"].files.len(), 1);
    }

    #[test]
    fn test_chapter_insert_deeply_nested() {
        let mut chapter = Chapter::new();
        chapter.insert(Path::new("src/utils/helpers.rs"));
        assert!(chapter.children.contains_key("src"));
        assert!(chapter.children["src"].children.contains_key("utils"));
        assert_eq!(chapter.children["src"].children["utils"].files.len(), 1);
    }

    #[test]
    fn test_chapter_has_content() {
        let mut chapter = Chapter::new();
        assert!(!chapter.has_content());

        chapter.insert(Path::new("file.rs"));
        assert!(chapter.has_content());
    }

    #[test]
    fn test_chapter_has_content_nested() {
        let mut chapter = Chapter::new();
        chapter.insert(Path::new("src/main.rs"));
        assert!(chapter.has_content());
        assert!(chapter.children["src"].has_content());
    }

    #[test]
    fn test_calculate_fence_basic() {
        assert_eq!(calculate_fence("no backticks"), "```");
        assert_eq!(calculate_fence("```rust\ncode\n```"), "````");
        assert_eq!(calculate_fence("````\ncode\n````"), "`````");
    }

    #[test]
    fn test_calculate_fence_single_backticks() {
        // Single backticks (inline code) should still result in minimum 3-backtick fence
        assert_eq!(calculate_fence("`inline code`"), "```");
    }

    #[test]
    fn test_calculate_fence_double_backticks() {
        // Double backticks should still result in minimum 3-backtick fence
        assert_eq!(calculate_fence("``double``"), "```");
    }
}

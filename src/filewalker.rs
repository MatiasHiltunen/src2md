use crate::writer::OUTPUT_MAGIC_BYTES;
use anyhow::Result;
use ignore::{DirEntry, WalkBuilder};
use log::{debug, trace, warn};
use memmap2::MmapOptions;
use std::collections::HashSet;
use std::ffi::OsStr;
use std::fs::File;
use std::path::{Path, PathBuf};

/// Lock file patterns that are excluded by default.
/// These files are typically auto-generated and not useful to include in documentation.
const LOCK_FILE_NAMES: &[&str] = &[
    "package-lock.json",
    "yarn.lock",
    "pnpm-lock.yaml",
    "bun.lockb",
    "Cargo.lock",
    "Gemfile.lock",
    "poetry.lock",
    "Pipfile.lock",
    "composer.lock",
    "mix.lock",
    "pubspec.lock",
    "flake.lock",
    "go.sum",
    "shrinkwrap.yaml",
];

/// File extensions that indicate lock files.
const LOCK_FILE_EXTENSIONS: &[&str] = &["lock", "lockb"];

/// Collects all files from the project root, applying ignore filters and specific path constraints.
///
/// # Arguments
///
/// * `project_root` - The root directory to start walking from
/// * `ignore_file` - Optional path to a custom ignore file (e.g., `.gitignore`, `.src2md.ignore`)
/// * `specific_paths` - If non-empty, only files within these paths are included
/// * `output_path` - Optional path to the output file being written (will be excluded to prevent race conditions)
/// * `extensions` - If non-empty, only files with these extensions are included
///
/// # Returns
///
/// A vector of `DirEntry` items representing all matching files.
///
/// # Default Exclusions
///
/// The following are always excluded:
/// - Hidden files and directories (starting with `.`)
/// - Lock files (package-lock.json, yarn.lock, Cargo.lock, etc.)
/// - The explicit `output_path` if provided
/// - Any file that starts with the src2md magic header
pub fn collect_files(
    project_root: &Path,
    ignore_file: Option<&PathBuf>,
    specific_paths: &HashSet<PathBuf>,
    output_path: Option<&PathBuf>,
    extensions: &HashSet<String>,
) -> Result<Vec<DirEntry>> {
    let mut builder = WalkBuilder::new(project_root);

    // Configure walker to skip hidden files/directories
    builder.hidden(true).ignore(false);

    // If a user-provided ignore file exists, use it
    if let Some(ignore_path) = ignore_file {
        debug!("Using ignore file: {}", ignore_path.display());
        builder.add_ignore(ignore_path);
    }

    // Canonicalize output path for reliable comparison
    let canonical_output = output_path.and_then(|p| p.canonicalize().ok());
    if let Some(ref out) = canonical_output {
        debug!("Excluding output file: {}", out.display());
    }

    if !extensions.is_empty() {
        debug!("Filtering by extensions: {:?}", extensions);
    }

    let walker = builder.build();
    let mut entries = Vec::new();
    let mut skipped_hidden = 0;
    let mut skipped_lock = 0;
    let mut skipped_outputs = 0;
    let mut skipped_extensions = 0;

    for result in walker {
        match result {
            Ok(entry) => {
                let path = entry.path();

                if !path.is_file() {
                    continue;
                }

                // Skip hidden files (the walker with hidden(true) should skip hidden dirs)
                if is_hidden(&entry) {
                    trace!("Skipping hidden file: {}", path.display());
                    skipped_hidden += 1;
                    continue;
                }

                // Skip lock files
                if is_lock_file(path) {
                    trace!("Skipping lock file: {}", path.display());
                    skipped_lock += 1;
                    continue;
                }

                // Check extension filter
                if !extensions.is_empty() && !has_matching_extension(path, extensions) {
                    trace!("Skipping file (extension filter): {}", path.display());
                    skipped_extensions += 1;
                    continue;
                }

                // Check specific paths filter
                if !specific_paths.is_empty() && !is_in_specific_paths(path, specific_paths) {
                    continue;
                }

                // Explicitly skip the output file by path (prevents race condition)
                if let Some(ref canonical_out) = canonical_output
                    && let Ok(canonical_path) = path.canonicalize()
                    && canonical_path == *canonical_out
                {
                    trace!("Skipping output file by path: {}", path.display());
                    skipped_outputs += 1;
                    continue;
                }

                // Check if file is a previous src2md output by reading its header
                if is_src2md_output(path) {
                    debug!("Skipping src2md output file: {}", path.display());
                    skipped_outputs += 1;
                    continue;
                }

                entries.push(entry);
            }
            Err(err) => {
                warn!("Error walking path: {err}");
            }
        }
    }

    // Log summary of skipped files
    if skipped_hidden > 0 {
        debug!("Skipped {} hidden file(s)", skipped_hidden);
    }
    if skipped_lock > 0 {
        debug!("Skipped {} lock file(s)", skipped_lock);
    }
    if skipped_outputs > 0 {
        debug!(
            "Skipped {} src2md output file(s) to prevent self-inclusion",
            skipped_outputs
        );
    }
    if skipped_extensions > 0 {
        debug!(
            "Skipped {} file(s) not matching extension filter",
            skipped_extensions
        );
    }

    debug!("Collected {} files", entries.len());
    Ok(entries)
}

/// Checks if a file is a src2md output by reading its magic header.
///
/// This uses memory-mapped I/O to efficiently read just the first few bytes
/// without loading the entire file into memory.
fn is_src2md_output(path: &Path) -> bool {
    let file = match File::open(path) {
        Ok(f) => f,
        Err(_) => return false,
    };

    // Get file metadata to check size
    let metadata = match file.metadata() {
        Ok(m) => m,
        Err(_) => return false,
    };

    // File must be at least as long as the magic header
    if metadata.len() < OUTPUT_MAGIC_BYTES.len() as u64 {
        return false;
    }

    // Memory-map just enough to check the header
    // SAFETY: We only read from the memory-mapped region and the file
    // remains open for the duration of the check.
    let mmap = match unsafe { MmapOptions::new().len(OUTPUT_MAGIC_BYTES.len()).map(&file) } {
        Ok(m) => m,
        Err(_) => return false,
    };

    mmap[..] == OUTPUT_MAGIC_BYTES[..]
}

/// Checks if a file itself is hidden (filename starts with a dot).
/// This only checks the filename, not the full path, since we filter
/// directories during the walk and shouldn't check parent directories
/// outside the project root.
fn is_hidden(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .is_some_and(|s| s.starts_with('.'))
}

/// Checks if a file is a lock file based on its name or extension.
fn is_lock_file(path: &Path) -> bool {
    let file_name = path.file_name().and_then(OsStr::to_str).unwrap_or("");

    // Check exact filename matches
    if LOCK_FILE_NAMES.contains(&file_name) {
        return true;
    }

    // Check extension
    if let Some(ext) = path.extension().and_then(OsStr::to_str)
        && LOCK_FILE_EXTENSIONS
            .iter()
            .any(|&e| e == ext.to_lowercase())
    {
        return true;
    }

    false
}

/// Checks if a file has an extension matching the provided set.
fn has_matching_extension(path: &Path, extensions: &HashSet<String>) -> bool {
    path.extension()
        .and_then(OsStr::to_str)
        .map(|ext| extensions.contains(&ext.to_lowercase()))
        .unwrap_or(false)
}

/// Checks if a given path is part of the explicitly included paths.
///
/// If the specific path is a file, it must match exactly.
/// If it's a directory, the path must be a descendant of that directory.
fn is_in_specific_paths(path: &Path, specific_paths: &HashSet<PathBuf>) -> bool {
    specific_paths.iter().any(|p| {
        if p.is_file() {
            path == p
        } else {
            path.starts_with(p)
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::writer::OUTPUT_MAGIC_HEADER;
    use std::fs;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_collect_files_basic() -> Result<()> {
        let temp_dir = tempdir()?;
        let root = temp_dir.path();

        fs::write(root.join("file1.rs"), "// rust")?;
        fs::write(root.join("file2.txt"), "text")?;

        let entries = collect_files(root, None, &HashSet::new(), None, &HashSet::new())?;

        assert_eq!(entries.len(), 2);
        Ok(())
    }

    #[test]
    fn test_collect_files_ignores_hidden() -> Result<()> {
        let temp_dir = tempdir()?;
        let root = temp_dir.path();

        fs::write(root.join("visible.rs"), "// visible")?;
        fs::write(root.join(".hidden.rs"), "// hidden")?;

        let entries = collect_files(root, None, &HashSet::new(), None, &HashSet::new())?;

        assert_eq!(entries.len(), 1);
        assert!(entries[0].path().to_string_lossy().contains("visible"));
        Ok(())
    }

    #[test]
    fn test_collect_files_ignores_hidden_directories() -> Result<()> {
        let temp_dir = tempdir()?;
        let root = temp_dir.path();

        fs::write(root.join("visible.rs"), "// visible")?;

        // Create hidden directory with files
        let hidden_dir = root.join(".hidden");
        fs::create_dir_all(&hidden_dir)?;
        fs::write(hidden_dir.join("secret.rs"), "// secret")?;

        let entries = collect_files(root, None, &HashSet::new(), None, &HashSet::new())?;

        assert_eq!(entries.len(), 1);
        assert!(entries[0].path().to_string_lossy().contains("visible"));
        Ok(())
    }

    #[test]
    fn test_collect_files_ignores_lock_files() -> Result<()> {
        let temp_dir = tempdir()?;
        let root = temp_dir.path();

        fs::write(root.join("source.rs"), "// source")?;
        fs::write(root.join("package-lock.json"), "{}")?;
        fs::write(root.join("yarn.lock"), "")?;
        fs::write(root.join("Cargo.lock"), "")?;
        fs::write(root.join("something.lock"), "")?;

        let entries = collect_files(root, None, &HashSet::new(), None, &HashSet::new())?;

        assert_eq!(entries.len(), 1);
        assert!(entries[0].path().to_string_lossy().contains("source.rs"));
        Ok(())
    }

    #[test]
    fn test_collect_files_ignores_nested_lock_files() -> Result<()> {
        let temp_dir = tempdir()?;
        let root = temp_dir.path();

        fs::write(root.join("source.rs"), "// source")?;

        let subdir = root.join("packages/frontend");
        fs::create_dir_all(&subdir)?;
        fs::write(subdir.join("app.ts"), "// app")?;
        fs::write(subdir.join("package-lock.json"), "{}")?;

        let entries = collect_files(root, None, &HashSet::new(), None, &HashSet::new())?;

        assert_eq!(entries.len(), 2);
        let paths: Vec<_> = entries.iter().map(|e| e.path().to_path_buf()).collect();
        assert!(paths
            .iter()
            .any(|p| p.to_string_lossy().contains("source.rs")));
        assert!(paths.iter().any(|p| p.to_string_lossy().contains("app.ts")));
        assert!(!paths
            .iter()
            .any(|p| p.to_string_lossy().contains("package-lock")));
        Ok(())
    }

    #[test]
    fn test_collect_files_with_extension_filter() -> Result<()> {
        let temp_dir = tempdir()?;
        let root = temp_dir.path();

        fs::write(root.join("main.rs"), "// rust")?;
        fs::write(root.join("app.ts"), "// typescript")?;
        fs::write(root.join("style.css"), "/* css */")?;
        fs::write(root.join("readme.md"), "# readme")?;

        let mut extensions = HashSet::new();
        extensions.insert("rs".to_string());
        extensions.insert("ts".to_string());

        let entries = collect_files(root, None, &HashSet::new(), None, &extensions)?;

        assert_eq!(entries.len(), 2);
        let paths: Vec<_> = entries.iter().map(|e| e.path().to_path_buf()).collect();
        assert!(paths
            .iter()
            .any(|p| p.to_string_lossy().contains("main.rs")));
        assert!(paths.iter().any(|p| p.to_string_lossy().contains("app.ts")));
        assert!(!paths
            .iter()
            .any(|p| p.to_string_lossy().contains("style.css")));
        assert!(!paths
            .iter()
            .any(|p| p.to_string_lossy().contains("readme.md")));
        Ok(())
    }

    #[test]
    fn test_collect_files_extension_filter_case_insensitive() -> Result<()> {
        let temp_dir = tempdir()?;
        let root = temp_dir.path();

        // Use different base names to avoid case-insensitive filesystem issues
        fs::write(root.join("uppercase.RS"), "// rust uppercase")?;
        fs::write(root.join("lowercase.rs"), "// rust lowercase")?;
        fs::write(root.join("mixed.Rs"), "// rust mixed")?;
        fs::write(root.join("other.txt"), "not included")?;

        let mut extensions = HashSet::new();
        extensions.insert("rs".to_string());

        let entries = collect_files(root, None, &HashSet::new(), None, &extensions)?;

        // Should include all .rs files regardless of extension case
        assert_eq!(entries.len(), 3);
        Ok(())
    }

    #[test]
    fn test_collect_files_with_specific_paths() -> Result<()> {
        let temp_dir = tempdir()?;
        let root = temp_dir.path();

        fs::write(root.join("included.rs"), "// included")?;
        fs::write(root.join("excluded.rs"), "// excluded")?;

        let mut specific = HashSet::new();
        specific.insert(root.join("included.rs"));

        let entries = collect_files(root, None, &specific, None, &HashSet::new())?;

        assert_eq!(entries.len(), 1);
        assert!(entries[0].path().to_string_lossy().contains("included"));
        Ok(())
    }

    #[test]
    fn test_collect_files_with_subdirectory() -> Result<()> {
        let temp_dir = tempdir()?;
        let root = temp_dir.path();

        let subdir = root.join("src");
        fs::create_dir_all(&subdir)?;
        fs::write(subdir.join("main.rs"), "// main")?;
        fs::write(root.join("other.rs"), "// other")?;

        let mut specific = HashSet::new();
        specific.insert(subdir);

        let entries = collect_files(root, None, &specific, None, &HashSet::new())?;

        assert_eq!(entries.len(), 1);
        assert!(entries[0].path().to_string_lossy().contains("main.rs"));
        Ok(())
    }

    #[test]
    fn test_collect_files_excludes_output_path() -> Result<()> {
        let temp_dir = tempdir()?;
        let root = temp_dir.path();

        fs::write(root.join("source.rs"), "// source")?;
        let output_path = root.join("output.md");
        fs::write(&output_path, "# Output")?;

        let entries = collect_files(root, None, &HashSet::new(), Some(&output_path), &HashSet::new())?;

        assert_eq!(entries.len(), 1);
        assert!(entries[0].path().to_string_lossy().contains("source.rs"));
        Ok(())
    }

    #[test]
    fn test_collect_files_excludes_src2md_output() -> Result<()> {
        let temp_dir = tempdir()?;
        let root = temp_dir.path();

        fs::write(root.join("source.rs"), "// source")?;

        // Create a file that looks like a previous src2md output
        let mut output_file = fs::File::create(root.join("previous_output.md"))?;
        output_file.write_all(OUTPUT_MAGIC_HEADER.as_bytes())?;
        output_file.write_all(b"\n## file.rs\n\n```rust\n// code\n```\n")?;

        let entries = collect_files(root, None, &HashSet::new(), None, &HashSet::new())?;

        assert_eq!(entries.len(), 1);
        assert!(entries[0].path().to_string_lossy().contains("source.rs"));
        Ok(())
    }

    #[test]
    fn test_collect_files_excludes_nested_src2md_outputs() -> Result<()> {
        let temp_dir = tempdir()?;
        let root = temp_dir.path();

        // Create source files
        fs::write(root.join("source.rs"), "// source")?;

        // Create nested directories with src2md outputs
        let docs_dir = root.join("docs");
        fs::create_dir_all(&docs_dir)?;
        fs::write(docs_dir.join("readme.md"), "# Readme")?;

        // Create a src2md output in the nested directory
        let mut output_file = fs::File::create(docs_dir.join("generated.md"))?;
        output_file.write_all(OUTPUT_MAGIC_HEADER.as_bytes())?;
        output_file.write_all(b"\n## nested/file.rs\n\n```rust\n// nested\n```\n")?;

        let entries = collect_files(root, None, &HashSet::new(), None, &HashSet::new())?;

        // Should include source.rs and readme.md but not generated.md
        assert_eq!(entries.len(), 2);
        let paths: Vec<_> = entries.iter().map(|e| e.path().to_path_buf()).collect();
        assert!(paths
            .iter()
            .any(|p| p.to_string_lossy().contains("source.rs")));
        assert!(paths
            .iter()
            .any(|p| p.to_string_lossy().contains("readme.md")));
        assert!(!paths
            .iter()
            .any(|p| p.to_string_lossy().contains("generated.md")));
        Ok(())
    }

    #[test]
    fn test_is_src2md_output() -> Result<()> {
        let temp_dir = tempdir()?;
        let root = temp_dir.path();

        // File with magic header
        let mut output_file = fs::File::create(root.join("output.md"))?;
        output_file.write_all(OUTPUT_MAGIC_HEADER.as_bytes())?;
        output_file.write_all(b"content")?;
        drop(output_file);

        // Regular file
        fs::write(root.join("regular.md"), "# Regular markdown")?;

        // Empty file
        fs::write(root.join("empty.md"), "")?;

        // File with partial header
        fs::write(root.join("partial.md"), "<!-- src")?;

        assert!(is_src2md_output(&root.join("output.md")));
        assert!(!is_src2md_output(&root.join("regular.md")));
        assert!(!is_src2md_output(&root.join("empty.md")));
        assert!(!is_src2md_output(&root.join("partial.md")));

        Ok(())
    }

    #[test]
    fn test_is_lock_file() {
        assert!(is_lock_file(Path::new("package-lock.json")));
        assert!(is_lock_file(Path::new("yarn.lock")));
        assert!(is_lock_file(Path::new("Cargo.lock")));
        assert!(is_lock_file(Path::new("pnpm-lock.yaml")));
        assert!(is_lock_file(Path::new("something.lock")));
        assert!(is_lock_file(Path::new("/path/to/package-lock.json")));

        assert!(!is_lock_file(Path::new("main.rs")));
        assert!(!is_lock_file(Path::new("package.json")));
        assert!(!is_lock_file(Path::new("lockfile.txt")));
    }

    #[test]
    fn test_is_hidden() -> Result<()> {
        let temp_dir = tempdir()?;
        let root = temp_dir.path();

        // Create test files
        fs::write(root.join("visible.rs"), "// visible")?;
        fs::write(root.join(".hidden"), "secret")?;

        // Walk to get DirEntry objects
        let mut visible_found = false;
        let mut hidden_found = false;

        let walker = ignore::WalkBuilder::new(root).hidden(false).build();
        for result in walker {
            if let Ok(entry) = result {
                if entry.path().is_file() {
                    let file_name = entry.file_name().to_string_lossy();
                    if file_name == "visible.rs" {
                        assert!(!is_hidden(&entry));
                        visible_found = true;
                    } else if file_name == ".hidden" {
                        assert!(is_hidden(&entry));
                        hidden_found = true;
                    }
                }
            }
        }

        assert!(visible_found, "visible.rs should be found");
        assert!(hidden_found, ".hidden should be found (with hidden=false)");

        Ok(())
    }

    #[test]
    fn test_has_matching_extension() {
        let mut extensions = HashSet::new();
        extensions.insert("rs".to_string());
        extensions.insert("ts".to_string());

        assert!(has_matching_extension(Path::new("main.rs"), &extensions));
        assert!(has_matching_extension(Path::new("app.ts"), &extensions));
        assert!(has_matching_extension(Path::new("FILE.RS"), &extensions)); // case insensitive

        assert!(!has_matching_extension(Path::new("style.css"), &extensions));
        assert!(!has_matching_extension(
            Path::new("README"),
            &extensions
        )); // no extension
    }
}

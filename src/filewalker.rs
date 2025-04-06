use anyhow::Result;
use ignore::{DirEntry, WalkBuilder};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// Collects all files from the project root, applying ignore filters and specific path constraints.
pub fn collect_files(
    project_root: &Path,
    ignore_file: Option<&PathBuf>,
    specific_paths: &HashSet<PathBuf>,
) -> Result<Vec<DirEntry>> {
    let mut builder = WalkBuilder::new(project_root);

    // Respect hidden files unless user says otherwise
    builder.hidden(true).ignore(false);

    // If a user-provided or fallback ignore file exists, use it
    if let Some(ignore_path) = ignore_file {
        builder.add_ignore(ignore_path);
    } else {
        builder.filter_entry(|e| !is_hidden(e));
    }

    let walker = builder.build();
    let mut entries = Vec::new();

    for result in walker {
        match result {
            Ok(entry) => {
                let path = entry.path();

                if path.is_file()
                    && (specific_paths.is_empty() || is_in_specific_paths(path, specific_paths))
                {
                    entries.push(entry);
                }
            }
            Err(err) => {
                eprintln!("Error walking path: {err}");
            }
        }
    }

    Ok(entries)
}

/// Determines if a file/folder is hidden (starts with a dot)
fn is_hidden(entry: &DirEntry) -> bool {
    entry
        .path()
        .file_name()
        .and_then(|s| s.to_str())
        .map_or(false, |s| s.starts_with('.'))
}

/// Checks if a given path is part of the explicitly included paths
fn is_in_specific_paths(path: &Path, specific_paths: &HashSet<PathBuf>) -> bool {
    specific_paths.iter().any(|p| {
        if p.is_file() {
            path == p
        } else {
            path.starts_with(p)
        }
    })
}

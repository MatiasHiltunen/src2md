use anyhow::Result;
use ignore::{DirEntry, WalkBuilder};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

pub fn collect_files(
    project_root: &Path,
    ignore_file: Option<&PathBuf>,
    specific_paths: &HashSet<PathBuf>,
) -> Result<Vec<DirEntry>> {
    let mut builder = WalkBuilder::new(project_root);
    builder.hidden(true).ignore(false); // Ignore hidden files/folders

    if let Some(ignore_path) = ignore_file {
        builder.add_ignore(ignore_path);
    } else {
        builder.filter_entry(|e| !is_hidden(e));
    }

    let walker = builder.build();
    let mut entries = Vec::new();

    for result in walker {
        if let Ok(entry) = result {
            if entry.path().is_file()
                && (specific_paths.is_empty() || is_in_specific_paths(entry.path(), specific_paths))
            {
                entries.push(entry);
            }
        }
    }

    Ok(entries)
}

fn is_hidden(entry: &DirEntry) -> bool {
    entry
        .path()
        .file_name()
        .and_then(|s| s.to_str())
        .map_or(false, |s| s.starts_with('.'))
}

fn is_in_specific_paths(path: &Path, specific_paths: &HashSet<PathBuf>) -> bool {
    specific_paths.iter().any(|p| {
        if p.is_file() {
            p == path
        } else {
            path.starts_with(p)
        }
    })
}

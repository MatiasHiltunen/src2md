//! Git repository cloning support.
//!
//! This module provides functionality to clone git repositories into temporary
//! directories for processing by src2md. It is only available when the `git`
//! feature is enabled.
//!
//! # Example
//!
//! ```rust,ignore
//! use src2md::git::clone_repository;
//!
//! let (temp_dir, repo_path) = clone_repository("https://github.com/user/repo")?;
//! // repo_path points to the cloned repository
//! // temp_dir is dropped when it goes out of scope, cleaning up the clone
//! ```

use anyhow::{Context, Result};
use git2::{build::RepoBuilder, FetchOptions, RemoteCallbacks};
use log::{debug, info};
use std::path::PathBuf;
use tempfile::TempDir;

/// Result of cloning a repository.
///
/// Contains the temporary directory handle (which cleans up on drop) and
/// the path to the cloned repository root.
pub struct ClonedRepo {
    /// The temporary directory containing the clone.
    /// Dropping this will delete the cloned repository.
    pub temp_dir: TempDir,
    /// Path to the repository root within the temp directory.
    pub path: PathBuf,
}

impl ClonedRepo {
    /// Returns the path to the cloned repository.
    pub fn path(&self) -> &PathBuf {
        &self.path
    }
}

/// Clones a git repository from the given URL into a temporary directory.
///
/// # Arguments
///
/// * `url` - The git URL to clone (HTTPS or SSH)
/// * `branch` - Optional branch name to checkout (defaults to the default branch)
///
/// # Returns
///
/// A `ClonedRepo` containing the temporary directory and path to the clone.
/// The temporary directory is automatically cleaned up when `ClonedRepo` is dropped.
///
/// # Errors
///
/// Returns an error if:
/// - The URL is invalid
/// - The repository cannot be cloned (network error, auth failure, etc.)
/// - The temporary directory cannot be created
pub fn clone_repository(url: &str, branch: Option<&str>) -> Result<ClonedRepo> {
    info!("Cloning repository: {}", url);

    // Create a temporary directory for the clone
    let temp_dir = TempDir::new().context("Failed to create temporary directory for git clone")?;

    let clone_path = temp_dir.path().to_path_buf();
    debug!("Clone target: {}", clone_path.display());

    // Set up progress callbacks for verbose output
    let mut callbacks = RemoteCallbacks::new();
    callbacks.transfer_progress(|progress| {
        if progress.received_objects() == progress.total_objects() {
            debug!(
                "Resolving deltas: {}/{}",
                progress.indexed_deltas(),
                progress.total_deltas()
            );
        } else {
            debug!(
                "Receiving objects: {}/{} ({} bytes)",
                progress.received_objects(),
                progress.total_objects(),
                progress.received_bytes()
            );
        }
        true
    });

    // Configure fetch options
    let mut fetch_opts = FetchOptions::new();
    fetch_opts.remote_callbacks(callbacks);
    fetch_opts.depth(1); // Shallow clone for speed

    // Build and execute the clone
    let mut builder = RepoBuilder::new();
    builder.fetch_options(fetch_opts);

    if let Some(branch_name) = branch {
        debug!("Checking out branch: {}", branch_name);
        builder.branch(branch_name);
    }

    builder
        .clone(url, &clone_path)
        .with_context(|| format!("Failed to clone repository: {}", url))?;

    info!("Clone complete: {}", clone_path.display());

    Ok(ClonedRepo {
        temp_dir,
        path: clone_path,
    })
}

/// Extracts the repository name from a git URL.
///
/// # Examples
///
/// ```rust,ignore
/// assert_eq!(repo_name_from_url("https://github.com/user/repo.git"), Some("repo"));
/// assert_eq!(repo_name_from_url("https://github.com/user/repo"), Some("repo"));
/// assert_eq!(repo_name_from_url("git@github.com:user/repo.git"), Some("repo"));
/// ```
pub fn repo_name_from_url(url: &str) -> Option<String> {
    // Handle both HTTPS and SSH URLs
    let path = if url.contains("://") {
        // HTTPS URL: https://github.com/user/repo.git
        url.rsplit('/').next()?
    } else if url.contains(':') {
        // SSH URL: git@github.com:user/repo.git
        url.rsplit(':').next()?.rsplit('/').next()?
    } else {
        return None;
    };

    // Remove .git suffix if present
    let name = path.strip_suffix(".git").unwrap_or(path);

    if name.is_empty() {
        None
    } else {
        Some(name.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_repo_name_from_https_url() {
        assert_eq!(
            repo_name_from_url("https://github.com/user/myrepo.git"),
            Some("myrepo".to_string())
        );
        assert_eq!(
            repo_name_from_url("https://github.com/user/myrepo"),
            Some("myrepo".to_string())
        );
        assert_eq!(
            repo_name_from_url("https://gitlab.com/group/subgroup/project.git"),
            Some("project".to_string())
        );
    }

    #[test]
    fn test_repo_name_from_ssh_url() {
        assert_eq!(
            repo_name_from_url("git@github.com:user/myrepo.git"),
            Some("myrepo".to_string())
        );
        assert_eq!(
            repo_name_from_url("git@github.com:user/myrepo"),
            Some("myrepo".to_string())
        );
    }

    #[test]
    fn test_repo_name_invalid_url() {
        assert_eq!(repo_name_from_url("not-a-url"), None);
        assert_eq!(repo_name_from_url(""), None);
    }
}


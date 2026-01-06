use anyhow::Result;
use chrono::Utc;
use clap::{Arg, Command};
use std::collections::HashSet;
use std::path::PathBuf;

/// Configuration for src2md operations.
pub struct Config {
    /// Output markdown file path.
    pub output_path: PathBuf,
    /// Optional custom ignore file path.
    pub ignore_file: Option<PathBuf>,
    /// Specific files or directories to include (if empty, includes all).
    pub specific_paths: HashSet<PathBuf>,
    /// Root directory to process.
    pub project_root: PathBuf,
    /// If set, restore files from this markdown file instead of generating.
    #[cfg(feature = "restore")]
    pub restore_input: Option<PathBuf>,
    /// Target directory for restoration.
    #[cfg(feature = "restore")]
    pub restore_path: Option<PathBuf>,
    /// Verbosity level (0-3).
    pub verbosity: u8,
    /// Stop on first error if true.
    pub fail_fast: bool,
    /// File extensions to include (if empty, includes all non-excluded).
    /// Extensions should be lowercase without the leading dot (e.g., "rs", "ts", "js").
    pub extensions: HashSet<String>,
    /// Git repository URL to clone and process (requires `git` feature).
    #[cfg(feature = "git")]
    pub git_url: Option<String>,
    /// Git branch to checkout (requires `git` feature).
    #[cfg(feature = "git")]
    pub git_branch: Option<String>,
}

/// Parses command-line arguments and returns a Config.
#[allow(unused_mut)] // mut is needed when features are enabled
pub fn parse_args() -> Result<Config> {
    let mut cmd = Command::new("src2md")
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about("Collects code and text files into a single .md file or restores them back")
        .arg(
            Arg::new("output")
                .short('o')
                .long("output")
                .value_name("FILE")
                .help("Sets the output .md file path")
                .num_args(1),
        )
        .arg(
            Arg::new("ignore-file")
                .long("ignore-file")
                .value_name("FILE")
                .help("Path to ignore file (like .gitignore)")
                .num_args(1),
        )
        .arg(
            Arg::new("paths")
                .value_name("PATHS")
                .help("Specific files or directories to include")
                .num_args(1..),
        )
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .help("Enable verbose output (can be repeated: -v, -vv, -vvv)")
                .action(clap::ArgAction::Count),
        )
        .arg(
            Arg::new("ext")
                .short('e')
                .long("ext")
                .value_name("EXTENSIONS")
                .help("Only include files with these extensions (comma-separated, e.g., rs,ts,js)")
                .num_args(1),
        )
        .arg(
            Arg::new("fail-fast")
                .long("fail-fast")
                .help("Stop processing on first error instead of continuing")
                .action(clap::ArgAction::SetTrue),
        );

    // Add restore-specific arguments when the feature is enabled
    #[cfg(feature = "restore")]
    {
        cmd = cmd
            .arg(
                Arg::new("restore")
                    .long("restore")
                    .value_name("MARKDOWN")
                    .help("Restore files from a src2md Markdown file back to filesystem")
                    .conflicts_with_all(["output", "ignore-file", "paths", "ext"]),
            )
            .arg(
                Arg::new("restore-path")
                    .long("restore-path")
                    .value_name("DIR")
                    .help("Target directory to restore files into (preserves relative paths)")
                    .requires("restore"),
            );
    }

    // Add git-specific arguments when the feature is enabled
    #[cfg(feature = "git")]
    {
        let conflicts: &[&str] = if cfg!(feature = "restore") {
            &["restore", "paths"]
        } else {
            &["paths"]
        };
        cmd = cmd
            .arg(
                Arg::new("git")
                    .long("git")
                    .value_name("URL")
                    .help("Clone a git repository and generate markdown from it")
                    .conflicts_with_all(conflicts),
            )
            .arg(
                Arg::new("branch")
                    .long("branch")
                    .short('b')
                    .value_name("BRANCH")
                    .help("Git branch to checkout (default: repository's default branch)")
                    .requires("git"),
            );
    }

    let matches = cmd.get_matches();

    let verbosity = matches.get_count("verbose");

    // Parse extensions from comma-separated list
    let extensions: HashSet<String> = matches
        .get_one::<String>("ext")
        .map(|s| {
            s.split(',')
                .map(|ext| {
                    ext.trim()
                        .to_lowercase()
                        .trim_start_matches('.')
                        .to_string()
                })
                .filter(|ext| !ext.is_empty())
                .collect()
        })
        .unwrap_or_default();

    // Handle restore mode
    #[cfg(feature = "restore")]
    if let Some(md_path) = matches.get_one::<String>("restore") {
        let restore_path = matches.get_one::<String>("restore-path").map(PathBuf::from);
        return Ok(Config {
            output_path: PathBuf::new(),
            ignore_file: None,
            specific_paths: HashSet::new(),
            project_root: PathBuf::new(),
            restore_input: Some(PathBuf::from(md_path)),
            restore_path,
            verbosity,
            fail_fast: matches.get_flag("fail-fast"),
            extensions: HashSet::new(),
            #[cfg(feature = "git")]
            git_url: None,
            #[cfg(feature = "git")]
            git_branch: None,
        });
    }

    // Handle git mode
    #[cfg(feature = "git")]
    if let Some(git_url) = matches.get_one::<String>("git") {
        let git_branch = matches.get_one::<String>("branch").cloned();

        // Extract repo name for default output filename
        let repo_name =
            crate::git::repo_name_from_url(git_url).unwrap_or_else(|| "repo".to_string());
        let timestamp = Utc::now().timestamp();
        let default_filename = format!("{repo_name}_content_{timestamp}.md");

        let output_path = matches
            .get_one::<String>("output")
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                std::env::current_dir()
                    .unwrap_or_default()
                    .join(default_filename)
            });

        let ignore_file = matches.get_one::<String>("ignore-file").map(PathBuf::from);

        return Ok(Config {
            output_path,
            ignore_file,
            specific_paths: HashSet::new(),
            project_root: PathBuf::new(), // Will be set after cloning
            #[cfg(feature = "restore")]
            restore_input: None,
            #[cfg(feature = "restore")]
            restore_path: None,
            verbosity,
            fail_fast: matches.get_flag("fail-fast"),
            extensions,
            git_url: Some(git_url.clone()),
            git_branch,
        });
    }

    // Standard mode: process local directory
    let project_root = std::env::current_dir()?;
    let default_filename = {
        let folder_name = project_root
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("project");
        let timestamp = Utc::now().timestamp();
        format!("{folder_name}_content_{timestamp}.md")
    };

    let output_path = matches
        .get_one::<String>("output")
        .map(PathBuf::from)
        .unwrap_or_else(|| project_root.join(default_filename));

    let ignore_file = matches.get_one::<String>("ignore-file").map(PathBuf::from);

    let specific_paths: HashSet<_> = matches
        .get_many::<String>("paths")
        .map(|vals| vals.map(|s| project_root.join(s)).collect())
        .unwrap_or_default();

    Ok(Config {
        output_path,
        ignore_file,
        specific_paths,
        project_root,
        #[cfg(feature = "restore")]
        restore_input: None,
        #[cfg(feature = "restore")]
        restore_path: None,
        verbosity,
        fail_fast: matches.get_flag("fail-fast"),
        extensions,
        #[cfg(feature = "git")]
        git_url: None,
        #[cfg(feature = "git")]
        git_branch: None,
    })
}

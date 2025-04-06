use anyhow::Result;
use chrono::Utc;
use clap::{Arg, Command};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

pub struct Config {
    pub output_path: PathBuf,
    pub ignore_file: Option<PathBuf>,
    pub specific_paths: HashSet<PathBuf>,
    pub project_root: PathBuf,
}

pub fn parse_args() -> Result<Config> {
    let matches = Command::new("src2md")
        .version("0.1.0")
        .author("Matias Hiltunen")
        .about("Collects code and text files into a single .md file")
        .arg(
            Arg::new("output")
                .short('o')
                .long("output")
                .value_name("FILE")
                .help("Sets the output .md file path")
                .num_args(1),
        )
        .arg(
            Arg::new("ignore")
                .short('i')
                .long("ignore")
                .value_name("FILE")
                .help("Sets the ignore file path")
                .num_args(1),
        )
        .arg(
            Arg::new("paths")
                .value_name("PATHS")
                .help("Specific files or directories to include")
                .num_args(1..),
        )
        .get_matches();

    let project_root = std::env::current_dir()?;

    // Build dynamic default filename: {project}_content_{epoch}.md
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

    let ignore_file = matches.get_one::<String>("ignore").map(PathBuf::from);

    let specific_paths: HashSet<_> = matches
        .get_many::<String>("paths")
        .map(|vals| vals.map(|s| project_root.join(s)).collect())
        .unwrap_or_default();

    Ok(Config {
        output_path,
        ignore_file,
        specific_paths,
        project_root,
    })
}

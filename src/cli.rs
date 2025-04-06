use anyhow::Result;
use chrono::Utc;
use clap::{Arg, Command};
use std::collections::HashSet;
use std::path::PathBuf;

pub struct Config {
    pub output_path: PathBuf,
    pub ignore_file: Option<PathBuf>,
    pub specific_paths: HashSet<PathBuf>,
    pub project_root: PathBuf,
    pub extract_input: Option<PathBuf>,
    pub extract_path: Option<PathBuf>,
}

pub fn parse_args() -> Result<Config> {
    let matches = Command::new("src2md")
        .version("0.1.0")
        .author("Matias Hiltunen")
        .about("Collects code and text files into a single .md file or extracts them back")
        .arg(
            Arg::new("output")
                .short('o')
                .long("output")
                .value_name("FILE")
                .help("Sets the output .md file path")
                .num_args(1)
                .requires_if("extract", ""),
        )
        .arg(
            Arg::new("ignore")
                .short('i')
                .long("ignore")
                .value_name("FILE")
                .help("Sets the ignore file path")
                .num_args(1)
                .requires_if("extract", ""),
        )
        .arg(
            Arg::new("paths")
                .value_name("PATHS")
                .help("Specific files or directories to include")
                .num_args(1..)
                .requires_if("extract", ""),
        )
        .arg(
            Arg::new("extract")
                .long("extract")
                .value_name("MARKDOWN")
                .help("Extracts code files from a Markdown file")
                .conflicts_with_all(["output", "ignore", "paths"]),
        )
        .arg(
            Arg::new("extract-path")
                .long("extract-path")
                .value_name("DIR")
                .help("Target directory to extract files into (preserves relative paths)")
                .requires("extract"),
        )
        .get_matches();

    if let Some(md_path) = matches.get_one::<String>("extract") {
        let extract_path = matches.get_one::<String>("extract-path").map(PathBuf::from);
        return Ok(Config {
            output_path: PathBuf::new(),
            ignore_file: None,
            specific_paths: HashSet::new(),
            project_root: PathBuf::new(),
            extract_input: Some(PathBuf::from(md_path)),
            extract_path,
        });
    }

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
        extract_input: None,
        extract_path: None,
    })
}

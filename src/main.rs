use clap::builder::PossibleValue;
use clap::{Arg, Command};
use content_inspector::{inspect, ContentType};
use ignore::{DirEntry, WalkBuilder};
use memmap2::Mmap;
use std::collections::HashSet;
use std::ffi::OsStr;
use std::fs::{self, File};
use std::io::{self, BufWriter, Write};
use std::path::{Path, PathBuf};

mod tests;

fn main() -> io::Result<()> {
    let config = parse_args()?;
    setup_output_directory(&config)?;

    let walker = build_walker(&config)?;

    for result in walker {
        match result {
            Ok(entry) => process_entry(entry, &config)?,
            Err(err) => eprintln!("Error: {}", err),
        }
    }

    Ok(())
}

struct Config {
    project_root: PathBuf,
    output_path: PathBuf,
    select_only: HashSet<PathBuf>,
    group_by: Option<String>,
    ignore_file: Option<PathBuf>,
    specific_paths: HashSet<PathBuf>,
}

fn parse_args() -> io::Result<Config> {
    let matches = Command::new("src2md")
        .version("0.1.0")
        .author("Matias Hiltunen https://github.com/MatiasHiltunen")
        .about("Collects code and text files into .md files")
        .arg(
            Arg::new("output")
                .id("output")
                .short('o')
                .long("output")
                .value_name("FILE")
                .help("Sets the output file or folder path")
                .num_args(1),
        )
        .arg(
            Arg::new("Custom ignore file")
                .id("ignore")
                .short('f')
                .long("ignore-file")
                .value_name("FILE")
                .help("Sets the ignore file path, follows the same rules as .gitignore")
                .num_args(1),
        )
        .arg(
            Arg::new("paths")
                .value_name("PATHS")
                .help("Specific files or directories to include")
                .num_args(1..)
                .trailing_var_arg(true),
        )
        .arg(
            Arg::new("select-only")
                .short('s')
                .long("select-only")
                .help("Include only the specified files or folders (ignore rules still apply)")
                .num_args(1..),
        )
        .arg(
            Arg::new("group-by")
                .value_name("TYPE")
                .long("group-by")
                .short('g')
                .help("Groups output by 'file' or 'folder'")
                .num_args(1)
                .value_parser([
                    PossibleValue::new("file").help("Creates separate .md files to all files"),
                    PossibleValue::new("folder")
                        .help("Creates <folder_name>.md files to output directory"),
                ]),
        )
        .get_matches();

    let project_root = std::env::current_dir()?;

    let output_path = matches
        .get_one::<String>("output")
        .map(PathBuf::from)
        .unwrap_or_else(|| project_root.join("all_the_code.md"));

    let select_only: HashSet<PathBuf> = matches
        .get_many::<String>("select-only")
        .map(|vals| vals.map(|s| project_root.join(s)).collect())
        .unwrap_or_else(HashSet::new);

    let group_by = matches.get_one::<String>("group-by").map(|s| s.to_string());

    let ignore_file_path = matches.get_one::<String>("ignore").map(PathBuf::from);

    let ignore_file = if let Some(path) = ignore_file_path {
        Some(path)
    } else if project_root.join(".src2md.ignore").exists() {
        Some(project_root.join(".src2md.ignore"))
    } else if project_root.join(".gitignore").exists() {
        Some(project_root.join(".gitignore"))
    } else {
        None
    };

    let specified_paths: Vec<String> = matches
        .get_many::<String>("paths")
        .map(|vals| vals.map(|s| s.to_string()).collect())
        .unwrap_or_else(|| vec![project_root.to_str().unwrap().to_string()]);

    let specific_paths: HashSet<PathBuf> = if matches.contains_id("paths") {
        specified_paths
            .iter()
            .map(|p| project_root.join(p))
            .collect()
    } else {
        HashSet::new()
    };

     // Validate output path and group_by combinations
     if group_by.is_none() && output_path.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Output path cannot be a directory when --group-by is not specified",
        ));
    }

    if let Some(ref _gb) = group_by {
        if output_path.is_file() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Output path must be a directory when using --group-by",
            ));
        }
    }

    Ok(Config {
        project_root,
        output_path,
        select_only,
        group_by,
        ignore_file,
        specific_paths,
    })
}

fn setup_output_directory(config: &Config) -> io::Result<()> {
    if let Some(ref _group_by) = config.group_by {
        if !config.output_path.exists() {
            fs::create_dir_all(&config.output_path)?;
        }
    } else if let Some(parent) = config.output_path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)?;
        }
    }
    Ok(())
}
fn build_walker(
    config: &Config,
) -> io::Result<impl Iterator<Item = Result<DirEntry, ignore::Error>>> {
    let mut walker_builder = WalkBuilder::new(&config.project_root);
    walker_builder.hidden(false).ignore(false);

    if let Some(ref ignore_file) = config.ignore_file {
        walker_builder.add_ignore(ignore_file);
    } else {
        walker_builder.filter_entry(|e| !is_hidden(e));
    }

    Ok(walker_builder.build())
}

fn process_entry(entry: DirEntry, config: &Config) -> io::Result<()> {
    let path = entry.path();

    if path.is_dir() || (!config.select_only.is_empty() && !config.select_only.contains(path)) {
        return Ok(());
    }

    if !config.specific_paths.is_empty() && !is_in_specific_paths(path, &config.specific_paths) {
        return Ok(());
    }

    let rel_path = path.strip_prefix(&config.project_root).unwrap();
    let file = File::open(path)?;
    let mmap = unsafe { Mmap::map(&file)? };

    let sample_size = std::cmp::min(8192, mmap.len());
    let content_type = inspect(&mmap[..sample_size]);

    let output_file = match config.group_by.as_deref() {
        Some("file") => config
            .output_path
            .join(rel_path.file_name().unwrap())
            .with_extension("md"),
        Some("folder") => {
            let folder_name = rel_path.parent().unwrap_or_else(|| Path::new("root"));
            config.output_path.join(folder_name).join("output.md")
        }
        _ => {
            // When output path is a file, we should write to it directly
            config.output_path.clone()
        }
    };

    fs::create_dir_all(output_file.parent().unwrap())?;
    let mut writer = BufWriter::new(File::create(output_file)?);

    if content_type == ContentType::BINARY {
        writeln!(writer, "## {}\n", rel_path.display())?;
    } else {
        writeln!(writer, "## {}\n", rel_path.display())?;
        let lang_tag = get_language_tag(path);
        writeln!(writer, "```{}\n", lang_tag)?;
        writer.write_all(&mmap)?;
        writeln!(writer, "\n```\n")?;
    }

    Ok(())
}

fn is_hidden(entry: &DirEntry) -> bool {
    entry
        .path()
        .file_name()
        .and_then(OsStr::to_str)
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

fn get_language_tag(path: &Path) -> &'static str {
    match path
        .extension()
        .and_then(OsStr::to_str)
        .unwrap_or("")
        .to_lowercase()
        .as_str()
    {
        "rs" => "rust",
        "js" => "javascript",
        "jsx" => "jsx",
        "tsx" => "tsx",
        "ts" => "typescript",
        "py" => "python",
        "java" => "java",
        "c" => "c",
        "cpp" => "cpp",
        "h" => "c",
        "html" => "html",
        "css" => "css",
        "md" => "markdown",
        "json" => "json",
        "toml" => "toml",
        "yaml" | "yml" => "yaml",
        "xml" => "xml",
        _ => "",
    }
}

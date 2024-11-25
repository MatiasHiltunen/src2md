use clap::builder::PossibleValue;
use clap::{Arg, Command};
use content_inspector::{inspect, ContentType};
use ignore::{DirEntry, WalkBuilder};
use memmap2::Mmap;
use std::collections::HashSet;
use std::fs::{self, File};
use std::io::{self, BufWriter, Write};
use std::path::{Path, PathBuf};

mod tests;

fn main() -> io::Result<()> {
    let config = parse_args()?;
    prepare_output(&config)?;

    let walker = build_walker(&config)?;

    process_files(walker, &config)?;

    Ok(())
}

#[derive(Debug, Clone)]
struct Config {
    project_root: PathBuf,
    output_path: PathBuf,
    select_only: HashSet<PathBuf>,
    group_by: Option<String>,
    ignore_file: Option<PathBuf>,
    specific_paths: HashSet<PathBuf>,
    default_ignores: HashSet<String>,
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
                    PossibleValue::new("file").help("Creates separate .md files for all files"),
                    PossibleValue::new("folder")
                        .help("Creates <folder_name>.md files in output directory"),
                ]),
        )
        .get_matches();

    let project_root = std::env::current_dir()?.canonicalize()?;

    let output_path = matches
        .get_one::<String>("output")
        .map(PathBuf::from)
        .unwrap_or_else(|| project_root.join("all_the_code.md"));

    let output_path = output_path.canonicalize().unwrap_or(output_path);

    let select_only: HashSet<PathBuf> = matches
        .get_many::<String>("select-only")
        .map(|vals| {
            vals.filter_map(|s| {
                let path = project_root.join(s);
                path.canonicalize().ok()
            })
            .collect()
        })
        .unwrap_or_else(HashSet::new);

    let group_by = matches.get_one::<String>("group-by").map(|s| s.to_string());

    // Validate output path and group_by combinations
    if group_by.is_none() && output_path.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Output path cannot be a directory when --group-by is not specified",
        ));
    }

    if let Some(ref gb) = group_by {
        if output_path.is_file() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Output path must be a directory when using --group-by",
            ));
        }
    }

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
            .filter_map(|p| project_root.join(p).canonicalize().ok())
            .collect()
    } else {
        HashSet::new()
    };

    let default_ignores = HashSet::from([
        ".git".to_string(),
        "node_modules".to_string(),
        "target".to_string(),
    ]);

    Ok(Config {
        project_root,
        output_path,
        select_only,
        group_by,
        ignore_file,
        specific_paths,
        default_ignores,
    })
}

fn prepare_output(config: &Config) -> io::Result<()> {
    // Remove existing output
    if config.output_path.exists() {
        if config.output_path.is_file() {
            fs::remove_file(&config.output_path)?;
        } else if config.output_path.is_dir() {
            fs::remove_dir_all(&config.output_path)?;
        }
    }

    // Ensure output directory exists
    if let Some(ref group_by) = config.group_by {
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
    let mut builder = WalkBuilder::new(&config.project_root);
    builder.hidden(false).ignore(false);

    if let Some(ref ignore_file) = config.ignore_file {
        builder.add_ignore(ignore_file);
    }

    // Clone the necessary data to move into the closure
    let output_path = config.output_path.clone();
    let default_ignores = config.default_ignores.clone();

    builder.filter_entry(move |entry| {
        let path = match entry.path().canonicalize() {
            Ok(p) => p,
            Err(_) => return false,
        };

        // Exclude the output path itself
        if path == output_path || path.starts_with(&output_path) {
            return false;
        }

        // Exclude default ignores
        if let Some(file_name) = entry.file_name().to_str() {
            if default_ignores.contains(file_name) {
                return false;
            }
        } else {
            return false;
        }

        // Exclude hidden files/directories
        if is_hidden(&entry) {
            return false;
        }

        true
    });

    Ok(builder.build())
}

fn should_include(entry: &DirEntry, config: &Config) -> bool {
    let path = match entry.path().canonicalize() {
        Ok(p) => p,
        Err(_) => return false,
    };

    // Exclude the output path itself
    if path == config.output_path || path.starts_with(&config.output_path) {
        return false;
    }

    // Exclude default ignores
    if config
        .default_ignores
        .contains(&entry.file_name().to_string_lossy().to_string())
    {
        return false;
    }

    // Exclude hidden files/directories
    if is_hidden(&entry) {
        return false;
    }

    true
}

fn is_hidden(entry: &DirEntry) -> bool {
    let file_name = entry.file_name();
    if file_name.to_string_lossy().starts_with('.') {
        return true;
    }

    #[cfg(target_os = "windows")]
    {
        use std::os::windows::fs::MetadataExt;
        entry
            .metadata()
            .map(|meta| meta.file_attributes() & 0x2 != 0)
            .unwrap_or(false)
    }

    #[cfg(not(target_os = "windows"))]
    {
        false
    }
}

fn process_files(
    walker: impl Iterator<Item = Result<DirEntry, ignore::Error>>,
    config: &Config,
) -> io::Result<()> {
    for result in walker {
        let entry = match result {
            Ok(e) => e,
            Err(err) => {
                eprintln!("Error: {}", err);
                continue;
            }
        };

        let path = entry.path();
        if path.is_dir() {
            continue;
        }

        let canonical_path = match path.canonicalize() {
            Ok(p) => p,
            Err(_) => continue,
        };

        if !config.select_only.is_empty() && !config.select_only.contains(&canonical_path) {
            continue;
        }

        if !config.specific_paths.is_empty()
            && !is_in_specific_paths(&canonical_path, &config.specific_paths)
        {
            continue;
        }

        process_entry(&canonical_path, config)?;
    }

    Ok(())
}

fn process_entry(path: &Path, config: &Config) -> io::Result<()> {
    let rel_path = path.strip_prefix(&config.project_root).unwrap();

    let file = File::open(path)?;
    let mmap = unsafe { Mmap::map(&file)? };

    let sample_size = std::cmp::min(8192, mmap.len());
    let content_type = inspect(&mmap[..sample_size]);

    let output_file = match config.group_by.as_deref() {
        Some("file") => {
            let mut output_file = config.output_path.join(rel_path);
            output_file.set_extension("md");
            output_file
        }
        Some("folder") => {
            let folder_path = rel_path.parent().unwrap_or_else(|| Path::new("root"));
            let mut output_file = config.output_path.join(folder_path);
            output_file.set_extension("md");
            output_file
        }
        _ => config.output_path.clone(),
    };

    if let Some(parent) = output_file.parent() {
        fs::create_dir_all(parent)?;
    }

    let mut writer = BufWriter::new(File::create(&output_file)?);

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

fn is_in_specific_paths(path: &Path, specific_paths: &HashSet<PathBuf>) -> bool {
    specific_paths.iter().any(|p| path.starts_with(p))
}

fn get_language_tag(path: &Path) -> &'static str {
    match path
        .extension()
        .and_then(|s| s.to_str())
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

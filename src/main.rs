use clap::{Arg, Command};
use content_inspector::{inspect, ContentType};
use ignore::{DirEntry, WalkBuilder};
use memmap2::Mmap;
use std::collections::HashSet;
use std::ffi::OsStr;
use std::fs::File;
use std::io::{self, BufWriter, Write};
use std::path::{Path, PathBuf};

fn main() -> io::Result<()> {
    let matches = Command::new("src2md")
        .version("0.1.0")
        .author("Matias Hiltunen https://github.com/MatiasHiltunen")
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
                .num_args(1..)
                .trailing_var_arg(true),
        )
        .get_matches();

    // Determine the project root and output file path
    let project_root = std::env::current_dir()?;
    let output_file_path = matches
        .get_one::<String>("output")
        .map(PathBuf::from)
        .unwrap_or_else(|| project_root.join("all_the_code.md"));

    // Open the output file
    let output_file = File::create(&output_file_path)?;
    let mut writer = BufWriter::new(output_file);

    // Determine the ignore file path
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

    // Collect specified paths or default to project root
    let specified_paths: Vec<String> = matches
        .get_many::<String>("paths")
        .map(|vals| vals.map(|s| s.to_string()).collect())
        .unwrap_or_else(|| vec![project_root.to_str().unwrap().to_string()]);

    // Create a HashSet of specific paths if any are provided
    let specific_paths: HashSet<PathBuf> = if matches.contains_id("paths") {
        specified_paths
            .iter()
            .map(|p| project_root.join(p))
            .collect()
    } else {
        HashSet::new()
    };

    let mut walker_builder = WalkBuilder::new(&project_root);
    walker_builder.hidden(false).ignore(false);

    if let Some(ref ignore_file) = ignore_file {
        walker_builder.add_ignore(ignore_file);
    } else {
        walker_builder.filter_entry(|e| !is_hidden(e));
    }

    let walker = walker_builder.build();

    for result in walker {
        match result {
            Ok(entry) => {
                let path = entry.path();

                if path.is_dir() {
                    continue;
                }

                // If specific paths are provided, skip files not in the set
                if !specific_paths.is_empty() && !is_in_specific_paths(path, &specific_paths) {
                    continue;
                }

                let rel_path = path.strip_prefix(&project_root).unwrap();
                let file = File::open(rel_path)?;
                let mmap = unsafe { Mmap::map(&file)? };

                // Determine if the file is binary or text
                let sample_size = std::cmp::min(8192, mmap.len());
                let content_type = inspect(&mmap[..sample_size]);

                if content_type == ContentType::BINARY {
                    // Write only the path for binary files
                    writeln!(writer, "## {}\n", rel_path.display())?;
                } else {
                    // Write the path and content for text files
                    writeln!(writer, "## {}\n", rel_path.display())?;

                    // Get the language tag based on file extension
                    let lang_tag = get_language_tag(path);

                    // Write code block with language tag
                    writeln!(writer, "```{}\n", lang_tag)?;
                    writer.write_all(&mmap)?;
                    writeln!(writer, "\n```\n")?;
                }
            }
            Err(err) => eprintln!("Error: {}", err),
        }
    }

    Ok(())
}

fn is_hidden(entry: &DirEntry) -> bool {
    entry
        .path()
        .file_name()
        .and_then(OsStr::to_str)
        .map(|s| s.starts_with('.'))
        .unwrap_or(false)
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
        "ts" => "typescript",
        "tsx" => "tsx",
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

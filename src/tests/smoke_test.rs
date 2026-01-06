use src2md::{extract_from_markdown, Config, run_src2md};
use std::collections::HashSet;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use tempfile::tempdir;
use tokio::fs;

#[tokio::test]
async fn it_generates_markdown_output() -> anyhow::Result<()> {
    let temp_dir = tempdir()?;
    let root_path = temp_dir.path().to_path_buf();
    let src_file_path = root_path.join("example.rs");

    let mut file = File::create(&src_file_path)?;
    writeln!(file, "fn main() {{ println!(\"Hello, world!\"); }}")?;

    let output_path = root_path.join("output.md");

    let config = Config {
        output_path: output_path.clone(),
        ignore_file: None,
        specific_paths: HashSet::new(),
        project_root: root_path.clone(),
        extract_input: None,
        extract_path: None,
        verbosity: 0,
        fail_fast: true,
    };

    run_src2md(config).await?;

    let contents = fs::read_to_string(&output_path).await?;
    assert!(contents.contains("```rust"));
    assert!(contents.contains("fn main()"));

    Ok(())
}

#[tokio::test]
async fn it_handles_files_with_backticks() -> anyhow::Result<()> {
    let temp_dir = tempdir()?;
    let root_path = temp_dir.path().to_path_buf();
    let md_file_path = root_path.join("README.md");

    // Create a markdown file that contains triple backticks
    let mut file = File::create(&md_file_path)?;
    writeln!(file, "# Example\n\n```rust\nfn main() {{}}\n```")?;

    let output_path = root_path.join("output.md");

    let config = Config {
        output_path: output_path.clone(),
        ignore_file: None,
        specific_paths: HashSet::new(),
        project_root: root_path.clone(),
        extract_input: None,
        extract_path: None,
        verbosity: 0,
        fail_fast: true,
    };

    run_src2md(config).await?;

    let contents = fs::read_to_string(&output_path).await?;
    // Should use 4 backticks to wrap content with 3 backticks
    assert!(contents.contains("````"));

    Ok(())
}

#[tokio::test]
async fn it_roundtrips_files() -> anyhow::Result<()> {
    let temp_dir = tempdir()?;
    let root_path = temp_dir.path().to_path_buf();

    // Create source files
    let src_dir = root_path.join("src");
    std::fs::create_dir_all(&src_dir)?;

    let main_content = "fn main() {\n    println!(\"Hello!\");\n}";
    std::fs::write(src_dir.join("main.rs"), main_content)?;

    let lib_content = "pub fn add(a: i32, b: i32) -> i32 {\n    a + b\n}";
    std::fs::write(src_dir.join("lib.rs"), lib_content)?;

    // Generate markdown
    let output_path = root_path.join("output.md");
    let config = Config {
        output_path: output_path.clone(),
        ignore_file: None,
        specific_paths: HashSet::new(),
        project_root: root_path.clone(),
        extract_input: None,
        extract_path: None,
        verbosity: 0,
        fail_fast: true,
    };

    run_src2md(config).await?;

    // Extract to a new directory
    let extract_dir = root_path.join("extracted");
    extract_from_markdown(&output_path, Some(&extract_dir)).await?;

    // Verify roundtrip
    let extracted_main = fs::read_to_string(extract_dir.join("src/main.rs")).await?;
    assert_eq!(extracted_main, main_content);

    let extracted_lib = fs::read_to_string(extract_dir.join("src/lib.rs")).await?;
    assert_eq!(extracted_lib, lib_content);

    Ok(())
}

#[tokio::test]
async fn it_handles_specific_paths() -> anyhow::Result<()> {
    let temp_dir = tempdir()?;
    let root_path = temp_dir.path().to_path_buf();

    // Create multiple files
    std::fs::write(root_path.join("included.rs"), "// included")?;
    std::fs::write(root_path.join("excluded.rs"), "// excluded")?;

    let output_path = root_path.join("output.md");

    let mut specific_paths = HashSet::new();
    specific_paths.insert(root_path.join("included.rs"));

    let config = Config {
        output_path: output_path.clone(),
        ignore_file: None,
        specific_paths,
        project_root: root_path.clone(),
        extract_input: None,
        extract_path: None,
        verbosity: 0,
        fail_fast: true,
    };

    run_src2md(config).await?;

    let contents = fs::read_to_string(&output_path).await?;
    assert!(contents.contains("included.rs"));
    assert!(!contents.contains("excluded.rs"));

    Ok(())
}

#[tokio::test]
async fn it_marks_binary_files() -> anyhow::Result<()> {
    let temp_dir = tempdir()?;
    let root_path = temp_dir.path().to_path_buf();

    // Create a binary file (just some non-UTF8 bytes)
    let binary_content: Vec<u8> = vec![0x00, 0x01, 0x02, 0xFF, 0xFE, 0x89, 0x50, 0x4E, 0x47];
    std::fs::write(root_path.join("binary.bin"), &binary_content)?;

    // Create a text file too
    std::fs::write(root_path.join("text.txt"), "Hello, world!")?;

    let output_path = root_path.join("output.md");

    let config = Config {
        output_path: output_path.clone(),
        ignore_file: None,
        specific_paths: HashSet::new(),
        project_root: root_path.clone(),
        extract_input: None,
        extract_path: None,
        verbosity: 0,
        fail_fast: true,
    };

    run_src2md(config).await?;

    let contents = fs::read_to_string(&output_path).await?;
    assert!(contents.contains("binary.bin"));
    assert!(contents.contains("(binary file omitted)"));
    assert!(contents.contains("Hello, world!"));

    Ok(())
}

#[cfg(feature = "restore")]
use src2md::extract_from_markdown;
use src2md::{Config, OUTPUT_MAGIC_HEADER, run_src2md};
use std::collections::HashSet;
use std::fs::File;
use std::io::Write;
use tempfile::tempdir;
use tokio::fs;

/// Creates a basic Config for testing.
fn test_config(output_path: std::path::PathBuf, project_root: std::path::PathBuf) -> Config {
    Config {
        output_path,
        ignore_file: None,
        specific_paths: HashSet::new(),
        project_root,
        #[cfg(feature = "restore")]
        restore_input: None,
        #[cfg(feature = "restore")]
        restore_path: None,
        verbosity: 0,
        fail_fast: true,
        extensions: HashSet::new(),
        #[cfg(feature = "git")]
        git_url: None,
        #[cfg(feature = "git")]
        git_branch: None,
        #[cfg(feature = "mdbook")]
        mdbook_output: None,
    }
}

/// Creates a Config with specific paths.
fn test_config_with_paths(
    output_path: std::path::PathBuf,
    project_root: std::path::PathBuf,
    specific_paths: HashSet<std::path::PathBuf>,
) -> Config {
    Config {
        output_path,
        ignore_file: None,
        specific_paths,
        project_root,
        #[cfg(feature = "restore")]
        restore_input: None,
        #[cfg(feature = "restore")]
        restore_path: None,
        verbosity: 0,
        fail_fast: true,
        extensions: HashSet::new(),
        #[cfg(feature = "git")]
        git_url: None,
        #[cfg(feature = "git")]
        git_branch: None,
        #[cfg(feature = "mdbook")]
        mdbook_output: None,
    }
}

/// Creates a Config with extension filtering.
fn test_config_with_extensions(
    output_path: std::path::PathBuf,
    project_root: std::path::PathBuf,
    extensions: HashSet<String>,
) -> Config {
    Config {
        output_path,
        ignore_file: None,
        specific_paths: HashSet::new(),
        project_root,
        #[cfg(feature = "restore")]
        restore_input: None,
        #[cfg(feature = "restore")]
        restore_path: None,
        verbosity: 0,
        fail_fast: true,
        extensions,
        #[cfg(feature = "git")]
        git_url: None,
        #[cfg(feature = "git")]
        git_branch: None,
        #[cfg(feature = "mdbook")]
        mdbook_output: None,
    }
}

#[tokio::test]
async fn it_generates_markdown_output() -> anyhow::Result<()> {
    let temp_dir = tempdir()?;
    let root_path = temp_dir.path().to_path_buf();
    let src_file_path = root_path.join("example.rs");

    let mut file = File::create(&src_file_path)?;
    writeln!(file, "fn main() {{ println!(\"Hello, world!\"); }}")?;

    let output_path = root_path.join("output.md");
    let config = test_config(output_path.clone(), root_path);

    run_src2md(config).await?;

    let contents = fs::read_to_string(&output_path).await?;

    // Should have magic header
    assert!(contents.starts_with(OUTPUT_MAGIC_HEADER));

    // Should have the code
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
    let config = test_config(output_path.clone(), root_path);

    run_src2md(config).await?;

    let contents = fs::read_to_string(&output_path).await?;
    // Should use 4 backticks to wrap content with 3 backticks
    assert!(contents.contains("````"));

    Ok(())
}

#[cfg(feature = "restore")]
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
    let config = test_config(output_path.clone(), root_path.clone());

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

    let config = test_config_with_paths(output_path.clone(), root_path, specific_paths);

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
    let config = test_config(output_path.clone(), root_path);

    run_src2md(config).await?;

    let contents = fs::read_to_string(&output_path).await?;
    assert!(contents.contains("binary.bin"));
    assert!(contents.contains("(binary file omitted)"));
    assert!(contents.contains("Hello, world!"));

    Ok(())
}

#[tokio::test]
async fn it_excludes_previous_outputs() -> anyhow::Result<()> {
    let temp_dir = tempdir()?;
    let root_path = temp_dir.path().to_path_buf();

    // Create a source file
    std::fs::write(root_path.join("source.rs"), "// source code")?;

    // Create a previous src2md output file
    let previous_output = format!(
        "{}## old/file.rs\n\n```rust\n// old\n```\n",
        OUTPUT_MAGIC_HEADER
    );
    std::fs::write(root_path.join("previous_output.md"), previous_output)?;

    let output_path = root_path.join("new_output.md");
    let config = test_config(output_path.clone(), root_path);

    run_src2md(config).await?;

    let contents = fs::read_to_string(&output_path).await?;

    // Should contain source.rs
    assert!(contents.contains("source.rs"));
    assert!(contents.contains("// source code"));

    // Should NOT contain the previous output
    assert!(!contents.contains("previous_output.md"));
    assert!(!contents.contains("old/file.rs"));

    Ok(())
}

#[tokio::test]
async fn it_excludes_nested_outputs() -> anyhow::Result<()> {
    let temp_dir = tempdir()?;
    let root_path = temp_dir.path().to_path_buf();

    // Create a source file
    std::fs::write(root_path.join("source.rs"), "// source code")?;

    // Create a nested directory with a src2md output
    let docs_dir = root_path.join("docs");
    std::fs::create_dir_all(&docs_dir)?;
    std::fs::write(docs_dir.join("readme.md"), "# Normal readme")?;

    let nested_output = format!(
        "{}## nested/file.rs\n\n```rust\n// nested\n```\n",
        OUTPUT_MAGIC_HEADER
    );
    std::fs::write(docs_dir.join("generated.md"), nested_output)?;

    let output_path = root_path.join("output.md");
    let config = test_config(output_path.clone(), root_path);

    run_src2md(config).await?;

    let contents = fs::read_to_string(&output_path).await?;

    // Should contain source.rs and readme.md
    assert!(contents.contains("source.rs"));
    assert!(contents.contains("readme.md"));
    assert!(contents.contains("# Normal readme"));

    // Should NOT contain the nested output
    assert!(!contents.contains("generated.md"));
    assert!(!contents.contains("nested/file.rs"));

    Ok(())
}

#[tokio::test]
async fn it_can_rerun_in_same_directory() -> anyhow::Result<()> {
    let temp_dir = tempdir()?;
    let root_path = temp_dir.path().to_path_buf();

    // Create a source file
    std::fs::write(root_path.join("source.rs"), "// version 1")?;

    let output_path = root_path.join("output.md");

    // First run
    let config = test_config(output_path.clone(), root_path.clone());
    run_src2md(config).await?;

    let first_contents = fs::read_to_string(&output_path).await?;
    assert!(first_contents.contains("// version 1"));

    // Update source file
    std::fs::write(root_path.join("source.rs"), "// version 2")?;

    // Second run - should NOT include the previous output
    let config = test_config(output_path.clone(), root_path);
    run_src2md(config).await?;

    let second_contents = fs::read_to_string(&output_path).await?;

    // Should have new version
    assert!(second_contents.contains("// version 2"));

    // Should NOT have old version (would be there if output was included)
    assert!(!second_contents.contains("// version 1"));

    // Should NOT include itself
    assert!(!second_contents.contains("output.md"));

    Ok(())
}

#[tokio::test]
async fn it_excludes_output_being_written() -> anyhow::Result<()> {
    let temp_dir = tempdir()?;
    let root_path = temp_dir.path().to_path_buf();

    // Create a source file
    std::fs::write(root_path.join("source.rs"), "// source")?;

    // Create an existing output file at the target path (simulating overwrite)
    let output_path = root_path.join("output.md");
    std::fs::write(&output_path, "# Previous content")?;

    let config = test_config(output_path.clone(), root_path);
    run_src2md(config).await?;

    let contents = fs::read_to_string(&output_path).await?;

    // Should have new content
    assert!(contents.starts_with(OUTPUT_MAGIC_HEADER));
    assert!(contents.contains("source.rs"));

    // Should NOT include the output file itself
    assert!(!contents.contains("output.md"));
    assert!(!contents.contains("# Previous content"));

    Ok(())
}

#[tokio::test]
async fn it_excludes_lock_files() -> anyhow::Result<()> {
    let temp_dir = tempdir()?;
    let root_path = temp_dir.path().to_path_buf();

    // Create source files
    std::fs::write(root_path.join("main.rs"), "// main")?;
    std::fs::write(root_path.join("package.json"), r#"{"name": "test"}"#)?;

    // Create lock files that should be excluded
    std::fs::write(root_path.join("package-lock.json"), "{}")?;
    std::fs::write(root_path.join("yarn.lock"), "")?;
    std::fs::write(root_path.join("Cargo.lock"), "")?;
    std::fs::write(root_path.join("pnpm-lock.yaml"), "")?;
    std::fs::write(root_path.join("custom.lock"), "")?;

    let output_path = root_path.join("output.md");
    let config = test_config(output_path.clone(), root_path);

    run_src2md(config).await?;

    let contents = fs::read_to_string(&output_path).await?;

    // Should include source files
    assert!(contents.contains("main.rs"));
    assert!(contents.contains("package.json"));

    // Should NOT include lock files
    assert!(!contents.contains("package-lock.json"));
    assert!(!contents.contains("yarn.lock"));
    assert!(!contents.contains("Cargo.lock"));
    assert!(!contents.contains("pnpm-lock.yaml"));
    assert!(!contents.contains("custom.lock"));

    Ok(())
}

#[tokio::test]
async fn it_excludes_hidden_files_and_directories() -> anyhow::Result<()> {
    let temp_dir = tempdir()?;
    let root_path = temp_dir.path().to_path_buf();

    // Create visible files
    std::fs::write(root_path.join("visible.rs"), "// visible")?;

    // Create hidden files
    std::fs::write(root_path.join(".hidden"), "secret")?;
    std::fs::write(root_path.join(".env"), "SECRET=value")?;

    // Create hidden directory with files
    let hidden_dir = root_path.join(".git");
    std::fs::create_dir_all(&hidden_dir)?;
    std::fs::write(hidden_dir.join("config"), "[core]")?;

    // Create nested hidden directory
    let nested_hidden = root_path.join("src/.hidden");
    std::fs::create_dir_all(&nested_hidden)?;
    std::fs::write(nested_hidden.join("secret.rs"), "// secret")?;

    let output_path = root_path.join("output.md");
    let config = test_config(output_path.clone(), root_path);

    run_src2md(config).await?;

    let contents = fs::read_to_string(&output_path).await?;

    // Should include visible files
    assert!(contents.contains("visible.rs"));

    // Should NOT include hidden files or files in hidden directories
    assert!(!contents.contains(".hidden"));
    assert!(!contents.contains(".env"));
    assert!(!contents.contains(".git"));
    assert!(!contents.contains("config"));
    assert!(!contents.contains("secret"));

    Ok(())
}

#[tokio::test]
async fn it_filters_by_extension() -> anyhow::Result<()> {
    let temp_dir = tempdir()?;
    let root_path = temp_dir.path().to_path_buf();

    // Create files with different extensions
    std::fs::write(root_path.join("main.rs"), "// rust")?;
    std::fs::write(root_path.join("app.ts"), "// typescript")?;
    std::fs::write(root_path.join("index.js"), "// javascript")?;
    std::fs::write(root_path.join("style.css"), "/* css */")?;
    std::fs::write(root_path.join("readme.md"), "# readme")?;

    let output_path = root_path.join("output.md");

    let mut extensions = HashSet::new();
    extensions.insert("rs".to_string());
    extensions.insert("ts".to_string());

    let config = test_config_with_extensions(output_path.clone(), root_path, extensions);

    run_src2md(config).await?;

    let contents = fs::read_to_string(&output_path).await?;

    // Should include only .rs and .ts files
    assert!(contents.contains("main.rs"));
    assert!(contents.contains("// rust"));
    assert!(contents.contains("app.ts"));
    assert!(contents.contains("// typescript"));

    // Should NOT include other files
    assert!(!contents.contains("index.js"));
    assert!(!contents.contains("style.css"));
    assert!(!contents.contains("readme.md"));

    Ok(())
}

#[tokio::test]
async fn it_filters_extension_case_insensitive() -> anyhow::Result<()> {
    let temp_dir = tempdir()?;
    let root_path = temp_dir.path().to_path_buf();

    // Create files with mixed-case extensions
    std::fs::write(root_path.join("file1.RS"), "// uppercase")?;
    std::fs::write(root_path.join("file2.Rs"), "// mixed")?;
    std::fs::write(root_path.join("file3.rs"), "// lowercase")?;

    let output_path = root_path.join("output.md");

    let mut extensions = HashSet::new();
    extensions.insert("rs".to_string());

    let config = test_config_with_extensions(output_path.clone(), root_path, extensions);

    run_src2md(config).await?;

    let contents = fs::read_to_string(&output_path).await?;

    // Should include all .rs files regardless of case
    assert!(contents.contains("// uppercase"));
    assert!(contents.contains("// mixed"));
    assert!(contents.contains("// lowercase"));

    Ok(())
}

// Git feature tests (only compiled when git feature is enabled)
#[cfg(feature = "git")]
mod git_tests {
    use src2md::git::repo_name_from_url;

    #[test]
    fn test_repo_name_extraction() {
        assert_eq!(
            repo_name_from_url("https://github.com/user/repo.git"),
            Some("repo".to_string())
        );
        assert_eq!(
            repo_name_from_url("https://github.com/user/repo"),
            Some("repo".to_string())
        );
        assert_eq!(
            repo_name_from_url("git@github.com:user/repo.git"),
            Some("repo".to_string())
        );
    }
}

// mdbook feature tests (only compiled when mdbook feature is enabled)
#[cfg(feature = "mdbook")]
mod mdbook_tests {
    use src2md::filewalker::collect_files;
    use src2md::generate_mdbook;
    use std::collections::HashSet;
    use tempfile::tempdir;
    use tokio::fs;

    #[tokio::test]
    async fn it_generates_mdbook_structure() -> anyhow::Result<()> {
        let temp_dir = tempdir()?;
        let root_path = temp_dir.path().to_path_buf();

        // Create source files
        std::fs::write(root_path.join("README.md"), "# Project README")?;

        let src_dir = root_path.join("src");
        std::fs::create_dir_all(&src_dir)?;
        std::fs::write(src_dir.join("main.rs"), "fn main() {}")?;
        std::fs::write(src_dir.join("lib.rs"), "pub fn hello() {}")?;

        // Output directory
        let output_dir = root_path.join("book");

        // Collect files and generate mdbook
        let entries = collect_files(&root_path, None, &HashSet::new(), None, &HashSet::new())?;
        generate_mdbook(&entries, &root_path, &output_dir).await?;

        // Verify SUMMARY.md exists
        let summary_path = output_dir.join("SUMMARY.md");
        assert!(summary_path.exists(), "SUMMARY.md should exist");

        let summary = fs::read_to_string(&summary_path).await?;
        assert!(summary.contains("# Summary"), "Should have Summary header");
        assert!(
            summary.contains("[Introduction]"),
            "Should have Introduction link"
        );
        assert!(summary.contains("[src]"), "Should have src chapter");

        // Verify introduction.md (root files)
        let intro_path = output_dir.join("introduction.md");
        assert!(intro_path.exists(), "introduction.md should exist");
        let intro = fs::read_to_string(&intro_path).await?;
        assert!(
            intro.contains("# Introduction"),
            "Should have Introduction header"
        );
        assert!(
            intro.contains("README.md"),
            "Should contain README.md section"
        );

        // Verify src.md (src folder files)
        let src_md_path = output_dir.join("src.md");
        assert!(src_md_path.exists(), "src.md should exist");
        let src_md = fs::read_to_string(&src_md_path).await?;
        assert!(src_md.contains("## main.rs"), "Should have main.rs section");
        assert!(src_md.contains("## lib.rs"), "Should have lib.rs section");
        assert!(
            src_md.contains("fn main()"),
            "Should contain main.rs content"
        );

        Ok(())
    }

    #[tokio::test]
    async fn it_handles_nested_directories() -> anyhow::Result<()> {
        let temp_dir = tempdir()?;
        let root_path = temp_dir.path().to_path_buf();

        // Create nested structure
        let utils_dir = root_path.join("src").join("utils");
        std::fs::create_dir_all(&utils_dir)?;
        std::fs::write(root_path.join("src").join("main.rs"), "// main")?;
        std::fs::write(utils_dir.join("helpers.rs"), "// helpers")?;

        let output_dir = root_path.join("book");

        let entries = collect_files(&root_path, None, &HashSet::new(), None, &HashSet::new())?;
        generate_mdbook(&entries, &root_path, &output_dir).await?;

        // Verify nested chapter exists
        let summary = fs::read_to_string(output_dir.join("SUMMARY.md")).await?;
        assert!(summary.contains("[src]"), "Should have src chapter");
        assert!(summary.contains("[utils]"), "Should have utils sub-chapter");

        // Verify src/utils.md exists with proper nesting
        let utils_md = output_dir.join("src").join("utils.md");
        assert!(utils_md.exists(), "src/utils.md should exist");
        let utils_content = fs::read_to_string(&utils_md).await?;
        assert!(
            utils_content.contains("helpers.rs"),
            "Should have helpers.rs"
        );

        Ok(())
    }

    #[tokio::test]
    async fn it_handles_binary_files_in_mdbook() -> anyhow::Result<()> {
        let temp_dir = tempdir()?;
        let root_path = temp_dir.path().to_path_buf();

        // Create a binary file
        let binary_content: Vec<u8> = vec![0x00, 0x01, 0x02, 0xFF, 0xFE];
        std::fs::write(root_path.join("binary.bin"), &binary_content)?;

        let output_dir = root_path.join("book");

        let entries = collect_files(&root_path, None, &HashSet::new(), None, &HashSet::new())?;
        generate_mdbook(&entries, &root_path, &output_dir).await?;

        let intro = fs::read_to_string(output_dir.join("introduction.md")).await?;
        assert!(intro.contains("binary.bin"), "Should list binary file");
        assert!(
            intro.contains("(binary file omitted)"),
            "Should mark as binary"
        );

        Ok(())
    }
}

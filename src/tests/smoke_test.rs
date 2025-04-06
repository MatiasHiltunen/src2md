use src2md::{Config, run_src2md};
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
    };

    run_src2md(config).await?;

    let contents = fs::read_to_string(output_path).await?;
    assert!(contents.contains("```rust"));
    assert!(contents.contains("fn main()"));

    Ok(())
}

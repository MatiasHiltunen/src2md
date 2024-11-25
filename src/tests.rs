// tests.rs

#[cfg(test)]
mod tests {
    use assert_cmd::Command;
    use predicates::prelude::*;
    use std::fs::{self, File};
    use std::io::Write;
    use tempfile::tempdir;

        /// Test the default functionality: all files collected into a single `.md` file.
        #[test]
        fn test_default_output() {
            let temp_dir = tempdir().unwrap();
            let test_file_path = temp_dir.path().join("test.rs");
            fs::write(&test_file_path, "fn main() {}").unwrap();
    
            let output_file = temp_dir.path().join("output.md");
    
            Command::cargo_bin("src2md")
                .unwrap()
                .arg("-o")
                .arg(output_file.to_str().unwrap())
                .arg(temp_dir.path())
                .assert()
                .success();
    
            let output = fs::read_to_string(&output_file).unwrap();
    
            assert!(output.contains("## test.rs"));
            assert!(output.contains("```rust\nfn main() {}\n```\n") || output.contains("```rust\r\nfn main() {}\n```\r\n"));
        }
    
        /// Test `--select-only` to ensure only specified files are included.
        #[test]
        fn test_select_only() {
            let temp_dir = tempdir().unwrap();
            let file1_path = temp_dir.path().join("include.rs");
            let file2_path = temp_dir.path().join("exclude.rs");
    
            fs::write(&file1_path, "fn include() {}").unwrap();
            fs::write(&file2_path, "fn exclude() {}").unwrap();
    
            let output_file = temp_dir.path().join("output.md");
    
            Command::cargo_bin("src2md")
                .unwrap()
                .arg("-o")
                .arg(output_file.to_str().unwrap())
                .arg("--select-only")
                .arg(file1_path.to_str().unwrap())
                .arg(temp_dir.path())
                .assert()
                .success();
    
            let output = fs::read_to_string(&output_file).unwrap();
            assert!(output.contains("## include.rs"));
            assert!(output.contains("fn include() {}"));
            assert!(!output.contains("## exclude.rs"));
        }
    /// Test `--group-by file` to ensure each file has its own `.md` output.
    #[test]
    fn test_group_by_file() {
        let temp_dir = tempdir().unwrap();
        let file1_path = temp_dir.path().join("file1.rs");
        let file2_path = temp_dir.path().join("file2.rs");

        fs::write(&file1_path, "fn file1() {}").unwrap();
        fs::write(&file2_path, "fn file2() {}").unwrap();

        let output_dir = temp_dir.path().join("output");

        Command::cargo_bin("src2md")
            .unwrap()
            .arg("-o")
            .arg(output_dir.to_str().unwrap())
            .arg("--group-by")
            .arg("file")
            .arg(temp_dir.path())
            .assert()
            .success();

        let file1_output = fs::read_to_string(output_dir.join("file1.md")).unwrap();
        let file2_output = fs::read_to_string(output_dir.join("file2.md")).unwrap();

        assert!(file1_output.contains("## file1.rs"));
        assert!(file1_output.contains("fn file1() {}"));
        assert!(file2_output.contains("## file2.rs"));
        assert!(file2_output.contains("fn file2() {}"));
    }

    /// Test `--group-by folder` to ensure files are grouped by folder structure.
    #[test]
    fn test_group_by_folder() {
        let temp_dir = tempdir().unwrap();
        let folder = temp_dir.path().join("nested");
        fs::create_dir(&folder).unwrap();
        let file1_path = folder.join("nested_file.rs");

        fs::write(&file1_path, "fn nested_file() {}").unwrap();

        let output_dir = temp_dir.path().join("output");

        Command::cargo_bin("src2md")
            .unwrap()
            .arg("-o")
            .arg(output_dir.to_str().unwrap())
            .arg("--group-by")
            .arg("folder")
            .arg(temp_dir.path())
            .assert()
            .success();

        let folder_output = fs::read_to_string(output_dir.join("nested/output.md")).unwrap();

        assert!(folder_output.contains("## nested_file.rs"));
        assert!(folder_output.contains("fn nested_file() {}"));
    }

    /// Test binary file handling: should include only file path in output.
    #[test]
    fn test_binary_file_handling() {
        let temp_dir = tempdir().unwrap();
        let binary_file_path = temp_dir.path().join("binary_file.bin");

        let mut file = File::create(&binary_file_path).unwrap();
        file.write_all(&[0x00, 0xFF, 0xAA, 0x55]).unwrap();

        let output_file = temp_dir.path().join("output.md");

        Command::cargo_bin("src2md")
            .unwrap()
            .arg("-o")
            .arg(output_file.to_str().unwrap())
            .arg(temp_dir.path())
            .assert()
            .success();

        let output = fs::read_to_string(output_file).unwrap();
        assert!(output.contains("## binary_file.bin"));
        assert!(!output.contains("```"));
    }

    /// Test ignoring files using a custom ignore file.
    #[test]
    fn test_ignore_file() {
        let temp_dir = tempdir().unwrap();
        let file1_path = temp_dir.path().join("include.rs");
        let file2_path = temp_dir.path().join("exclude.rs");
        let ignore_file_path = temp_dir.path().join(".src2md.ignore");

        fs::write(&file1_path, "fn include() {}").unwrap();
        fs::write(&file2_path, "fn exclude() {}").unwrap();
        fs::write(&ignore_file_path, "exclude.rs").unwrap();

        let output_file = temp_dir.path().join("output.md");

        Command::cargo_bin("src2md")
            .unwrap()
            .arg("-o")
            .arg(output_file.to_str().unwrap())
            .arg(temp_dir.path())
            .assert()
            .success();

        let output = fs::read_to_string(output_file).unwrap();
        assert!(output.contains("## include.rs"));
        assert!(output.contains("fn include() {}"));
        assert!(!output.contains("## exclude.rs"));
    }

    /// Test specifying a custom ignore file via `--ignore-file`.
    #[test]
    fn test_custom_ignore_file() {
        let temp_dir = tempdir().unwrap();
        let file1_path = temp_dir.path().join("include.rs");
        let file2_path = temp_dir.path().join("exclude.rs");
        let custom_ignore_file_path = temp_dir.path().join("my_ignore.txt");

        fs::write(&file1_path, "fn include() {}").unwrap();
        fs::write(&file2_path, "fn exclude() {}").unwrap();
        fs::write(&custom_ignore_file_path, "exclude.rs").unwrap();

        let output_file = temp_dir.path().join("output.md");

        Command::cargo_bin("src2md")
            .unwrap()
            .arg("-o")
            .arg(output_file.to_str().unwrap())
            .arg("--ignore-file")
            .arg(custom_ignore_file_path.to_str().unwrap())
            .arg(temp_dir.path())
            .assert()
            .success();

        let output = fs::read_to_string(output_file).unwrap();
        assert!(output.contains("## include.rs"));
        assert!(output.contains("fn include() {}"));
        assert!(!output.contains("## exclude.rs"));
    }

    /// Test language detection for various file types.
    #[test]
    fn test_language_detection() {
        let temp_dir = tempdir().unwrap();
        let file1_path = temp_dir.path().join("script.py");
        let file2_path = temp_dir.path().join("markup.html");

        fs::write(&file1_path, "print('Hello, World!')").unwrap();
        fs::write(&file2_path, "<h1>Hello, World!</h1>").unwrap();

        let output_file = temp_dir.path().join("output.md");

        Command::cargo_bin("src2md")
            .unwrap()
            .arg("-o")
            .arg(output_file.to_str().unwrap())
            .arg(temp_dir.path())
            .assert()
            .success();

        let output = fs::read_to_string(output_file).unwrap();
        assert!(output.contains("## script.py"));
        assert!(output.contains("```python\nprint('Hello, World!')\n```\n"));
        assert!(output.contains("## markup.html"));
        assert!(output.contains("```html\n<h1>Hello, World!</h1>\n```\n"));
    }

    /// Test handling when no paths are specified (should use current directory).
    #[test]
    fn test_no_paths_specified() {
        let temp_dir = tempdir().unwrap();
        let current_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(&temp_dir).unwrap();

        let test_file_path = temp_dir.path().join("test.rs");
        fs::write(&test_file_path, "fn main() {}").unwrap();

        let output_file = temp_dir.path().join("output.md");

        Command::cargo_bin("src2md")
            .unwrap()
            .arg("-o")
            .arg("output.md")
            .assert()
            .success();

        let output = fs::read_to_string(output_file).unwrap();

        assert!(output.contains("## test.rs"));
        assert!(output.contains("```rust\nfn main() {}\n```\n"));

        // Restore the original current directory
        std::env::set_current_dir(current_dir).unwrap();
    }

    /// Test handling of hidden files.
    #[test]
    fn test_hidden_files() {
        let temp_dir = tempdir().unwrap();
        let hidden_file_path = temp_dir.path().join(".hidden.rs");
        fs::write(&hidden_file_path, "fn hidden() {}").unwrap();

        let output_file = temp_dir.path().join("output.md");

        // By default, hidden files should be ignored
        Command::cargo_bin("src2md")
            .unwrap()
            .arg("-o")
            .arg(output_file.to_str().unwrap())
            .arg(temp_dir.path())
            .assert()
            .success();

        let output = fs::read_to_string(&output_file).unwrap();
        assert!(!output.contains("## .hidden.rs"));

        // If we specify the hidden file explicitly, it should be included
        Command::cargo_bin("src2md")
            .unwrap()
            .arg("-o")
            .arg(output_file.to_str().unwrap())
            .arg("--select-only")
            .arg(hidden_file_path.to_str().unwrap())
            .arg(temp_dir.path())
            .assert()
            .success();

        let output = fs::read_to_string(output_file).unwrap();
        assert!(output.contains("## .hidden.rs"));
        assert!(output.contains("fn hidden() {}"));
    }

    /// Test invalid argument handling.
    #[test]
    fn test_invalid_arguments() {
        Command::cargo_bin("src2md")
            .unwrap()
            .arg("--invalid-arg")
            .assert()
            .failure();
    }

    /// Test when output path is a directory without `--group-by`.
    #[test]
    fn test_output_dir_without_group_by() {
        let temp_dir = tempdir().unwrap();
        let test_file_path = temp_dir.path().join("test.rs");
        fs::write(&test_file_path, "fn main() {}").unwrap();

        let output_dir = temp_dir.path().join("output_dir");
        fs::create_dir(&output_dir).unwrap();

        Command::cargo_bin("src2md")
            .unwrap()
            .arg("-o")
            .arg(output_dir.to_str().unwrap())
            .arg(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "Output path cannot be a directory when --group-by is not specified",
            ));
    }

    /// Test when output path is a file with `--group-by folder`.
    #[test]
    fn test_output_file_with_group_by_folder() {
        let temp_dir = tempdir().unwrap();
        let test_file_path = temp_dir.path().join("test.rs");
        fs::write(&test_file_path, "fn main() {}").unwrap();

        let output_file = temp_dir.path().join("output.md");

        Command::cargo_bin("src2md")
            .unwrap()
            .arg("-o")
            .arg(output_file.to_str().unwrap())
            .arg("--group-by")
            .arg("folder")
            .arg(temp_dir.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "Output path must be a directory when using --group-by",
            ));
    }

    /// Test when `--select-only` is used with a directory.
    #[test]
    fn test_select_only_directory() {
        let temp_dir = tempdir().unwrap();
        let dir_path = temp_dir.path().join("src");
        fs::create_dir(&dir_path).unwrap();

        let file1_path = dir_path.join("file1.rs");
        let file2_path = dir_path.join("file2.rs");
        fs::write(&file1_path, "fn file1() {}").unwrap();
        fs::write(&file2_path, "fn file2() {}").unwrap();

        let output_file = temp_dir.path().join("output.md");

        Command::cargo_bin("src2md")
            .unwrap()
            .arg("-o")
            .arg(output_file.to_str().unwrap())
            .arg("--select-only")
            .arg(dir_path.to_str().unwrap())
            .arg(temp_dir.path())
            .assert()
            .success();

        let output = fs::read_to_string(output_file).unwrap();
        assert!(output.contains("## file1.rs"));
        assert!(output.contains("fn file1() {}"));
        assert!(output.contains("## file2.rs"));
        assert!(output.contains("fn file2() {}"));
    }
}

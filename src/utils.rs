use std::ffi::OsStr;
use std::path::Path;

pub fn get_language_tag(path: &Path) -> &'static str {
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

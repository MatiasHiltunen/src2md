use std::ffi::OsStr;
use std::path::Path;

/// Maps file extensions to Markdown language tags for syntax highlighting.
///
/// Returns an empty string for unknown extensions, allowing the code block
/// to render without specific highlighting.
pub fn get_language_tag(path: &Path) -> &'static str {
    let ext = path
        .extension()
        .and_then(OsStr::to_str)
        .unwrap_or("")
        .to_lowercase();

    // Also check filename for extensionless files like Makefile, Dockerfile
    let filename = path
        .file_name()
        .and_then(OsStr::to_str)
        .unwrap_or("")
        .to_lowercase();

    // Check filename first for special cases
    match filename.as_str() {
        "dockerfile" => return "dockerfile",
        "makefile" | "gnumakefile" => return "makefile",
        "cmakelists.txt" => return "cmake",
        "rakefile" | "gemfile" => return "ruby",
        "vagrantfile" => return "ruby",
        "justfile" => return "just",
        ".gitignore" | ".gitattributes" | ".gitmodules" => return "gitignore",
        ".env" | ".env.local" | ".env.example" => return "dotenv",
        ".editorconfig" => return "editorconfig",
        "procfile" => return "procfile",
        _ => {}
    }

    match ext.as_str() {
        // Rust
        "rs" => "rust",

        // JavaScript / TypeScript
        "js" => "javascript",
        "mjs" => "javascript",
        "cjs" => "javascript",
        "jsx" => "jsx",
        "ts" => "typescript",
        "mts" => "typescript",
        "cts" => "typescript",
        "tsx" => "tsx",

        // Web
        "html" | "htm" => "html",
        "css" => "css",
        "scss" => "scss",
        "sass" => "sass",
        "less" => "less",
        "vue" => "vue",
        "svelte" => "svelte",
        "astro" => "astro",

        // Python
        "py" => "python",
        "pyi" => "python",
        "pyw" => "python",
        "pyx" => "cython",
        "pxd" => "cython",

        // Ruby
        "rb" => "ruby",
        "erb" => "erb",
        "rake" => "ruby",
        "gemspec" => "ruby",

        // Go
        "go" => "go",
        "mod" => "gomod",
        "sum" => "gosum",

        // Java / JVM
        "java" => "java",
        "kt" | "kts" => "kotlin",
        "scala" | "sc" => "scala",
        "groovy" | "gvy" | "gy" | "gsh" => "groovy",
        "gradle" => "gradle",
        "clj" | "cljs" | "cljc" | "edn" => "clojure",

        // C / C++
        "c" => "c",
        "h" => "c",
        "cpp" | "cc" | "cxx" | "c++" => "cpp",
        "hpp" | "hh" | "hxx" | "h++" => "cpp",

        // C# / .NET
        "cs" => "csharp",
        "fs" | "fsi" | "fsx" => "fsharp",
        "vb" => "vb",
        "csproj" | "fsproj" | "vbproj" | "sln" => "xml",

        // Systems
        "zig" => "zig",
        "nim" => "nim",
        "v" => "v",
        "odin" => "odin",
        "d" => "d",

        // Functional
        "hs" | "lhs" => "haskell",
        "ml" | "mli" => "ocaml",
        "ex" | "exs" => "elixir",
        "erl" | "hrl" => "erlang",
        "elm" => "elm",
        "purs" => "purescript",
        "rkt" => "racket",
        "scm" | "ss" => "scheme",
        "lisp" | "lsp" | "cl" => "lisp",

        // Shell
        "sh" => "bash",
        "bash" => "bash",
        "zsh" => "zsh",
        "fish" => "fish",
        "ps1" | "psm1" | "psd1" => "powershell",
        "bat" | "cmd" => "batch",

        // Config / Data
        "json" => "json",
        "jsonc" => "jsonc",
        "json5" => "json5",
        "yaml" | "yml" => "yaml",
        "toml" => "toml",
        "xml" => "xml",
        "xsd" | "xsl" | "xslt" => "xml",
        "ini" | "cfg" => "ini",
        "conf" => "conf",
        "properties" => "properties",

        // Markup / Documentation
        "md" | "markdown" => "markdown",
        "mdx" => "mdx",
        "rst" => "rst",
        "tex" | "latex" => "latex",
        "adoc" | "asciidoc" => "asciidoc",
        "org" => "org",
        "txt" => "text",

        // Database
        "sql" => "sql",
        "psql" => "sql",
        "mysql" => "sql",
        "pgsql" => "sql",
        "plsql" => "plsql",
        "prisma" => "prisma",

        // DevOps / Infrastructure
        "tf" | "tfvars" => "hcl",
        "hcl" => "hcl",
        "nix" => "nix",
        "dhall" => "dhall",

        // PHP
        "php" | "phtml" => "php",
        "blade.php" => "blade",

        // Swift / Apple
        "swift" => "swift",
        // Note: .m is ambiguous (Objective-C vs MATLAB). We default to Objective-C
        // since it's more common in mixed codebases. Use .mat for MATLAB data files.
        "m" | "mm" => "objectivec",

        // Perl
        "pl" | "pm" | "pod" => "perl",
        "perl" => "perl",

        // Lua
        "lua" => "lua",
        "luau" => "luau",

        // R
        "r" | "rmd" => "r",

        // Julia
        "jl" => "julia",

        // MATLAB / Octave (for .m files, see note above - defaults to Objective-C)
        "mat" => "matlab",

        // Assembly
        "asm" | "s" => "asm",
        "nasm" => "nasm",

        // Protocol / Schema
        "proto" => "protobuf",
        "graphql" | "gql" => "graphql",
        "thrift" => "thrift",
        "avsc" => "json",

        // Templates
        "ejs" => "ejs",
        "hbs" | "handlebars" => "handlebars",
        "mustache" => "mustache",
        "jinja" | "jinja2" | "j2" => "jinja",
        "liquid" => "liquid",
        "pug" | "jade" => "pug",
        "slim" => "slim",
        "haml" => "haml",

        // Misc
        "diff" | "patch" => "diff",
        "log" => "log",
        "csv" => "csv",
        "tsv" => "tsv",
        "lock" => "lock",
        "svg" => "svg",
        "wasm" | "wat" => "wasm",
        "glsl" | "vert" | "frag" => "glsl",
        "hlsl" => "hlsl",
        "cu" | "cuh" => "cuda",
        "sol" => "solidity",
        "cairo" => "cairo",
        "move" => "move",

        _ => "",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_common_extensions() {
        assert_eq!(get_language_tag(Path::new("main.rs")), "rust");
        assert_eq!(get_language_tag(Path::new("app.js")), "javascript");
        assert_eq!(get_language_tag(Path::new("style.css")), "css");
        assert_eq!(get_language_tag(Path::new("data.json")), "json");
        assert_eq!(get_language_tag(Path::new("config.yaml")), "yaml");
        assert_eq!(get_language_tag(Path::new("script.py")), "python");
    }

    #[test]
    fn test_case_insensitivity() {
        assert_eq!(get_language_tag(Path::new("file.RS")), "rust");
        assert_eq!(get_language_tag(Path::new("file.Js")), "javascript");
        assert_eq!(get_language_tag(Path::new("file.PY")), "python");
    }

    #[test]
    fn test_special_filenames() {
        assert_eq!(get_language_tag(Path::new("Dockerfile")), "dockerfile");
        assert_eq!(get_language_tag(Path::new("Makefile")), "makefile");
        assert_eq!(get_language_tag(Path::new("CMakeLists.txt")), "cmake");
        assert_eq!(get_language_tag(Path::new("Gemfile")), "ruby");
    }

    #[test]
    fn test_unknown_extension() {
        assert_eq!(get_language_tag(Path::new("file.xyz")), "");
        assert_eq!(get_language_tag(Path::new("noextension")), "");
    }

    #[test]
    fn test_typescript_variants() {
        assert_eq!(get_language_tag(Path::new("index.ts")), "typescript");
        assert_eq!(get_language_tag(Path::new("index.tsx")), "tsx");
        assert_eq!(get_language_tag(Path::new("index.mts")), "typescript");
    }

    #[test]
    fn test_shell_scripts() {
        assert_eq!(get_language_tag(Path::new("script.sh")), "bash");
        assert_eq!(get_language_tag(Path::new("script.bash")), "bash");
        assert_eq!(get_language_tag(Path::new("script.zsh")), "zsh");
        assert_eq!(get_language_tag(Path::new("script.ps1")), "powershell");
    }

    #[test]
    fn test_with_path() {
        assert_eq!(
            get_language_tag(&PathBuf::from("/some/path/to/file.rs")),
            "rust"
        );
        assert_eq!(
            get_language_tag(&PathBuf::from("relative/path/Dockerfile")),
            "dockerfile"
        );
    }
}

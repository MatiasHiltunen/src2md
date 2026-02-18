# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/)

This project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.8] - 2026-02-18

### Fixed

- Restore security: block path traversal and absolute-path writes when using `--restore-path`
- Restore parsing: ignore `##` headings inside fenced code blocks
- Restore fidelity: preserve leading whitespace in restored filenames
- mdbook output: always generate chapter files for chapters referenced from `SUMMARY.md`

## [0.1.7] - 2026-01-06

### Fixed

- Backtick fence calculation: Files with single/double inline backticks now correctly use minimum 3-backtick fences (was producing invalid 2-backtick fences)
- Git clone: Removed shallow clone to ensure all files are properly fetched
- OpenSSL cross-compilation: Use vendored OpenSSL for ARM64 Linux builds

## [0.1.6] - 2026-01-06

### Added

- `--mdbook <DIR>` flag to generate mdbook-compatible output
- `mdbook` feature flag for mdbook format output (enabled by default)
- Each folder becomes a chapter, each file becomes a section
- Nested folders become nested chapters with proper SUMMARY.md structure

## [0.1.5] - 2026-01-06

### Added

- `--ext` flag to filter files by extension (e.g., `--ext rs,ts,js`)
- Automatic exclusion of hidden files and directories
- Automatic exclusion of lock files (`Cargo.lock`, `package-lock.json`, `yarn.lock`, etc.)
- Automatic exclusion of previous src2md output files (detected by magic header)
- `--verbose` flag with multiple levels (`-v`, `-vv`, `-vvv`)
- `--fail-fast` flag to stop on first error
- Cross-platform release binaries (Linux, macOS, Windows)
- SHA256 checksums for release artifacts
- `restore` feature flag for restore/extract functionality (enabled by default)

### Changed

- Renamed `--extract` to `--restore` for clarity
- Renamed `-i, --ignore` to `--ignore-file` to avoid confusion with "include"
- Improved CI pipeline with cross-platform testing
- Added MSRV (Minimum Supported Rust Version) of 1.85
- All optional features (`git`, `restore`) are now enabled by default
- Library users can disable default features for minimal dependencies

### Fixed

- Race condition in file extraction when running tests in parallel

## [0.1.4] - 2025-12-XX

### Added

- Git repository cloning support via `--git` flag (requires `git` feature)
- Magic header to identify src2md output files
- Output file exclusion to prevent self-inclusion

### Fixed

- Fence matching for nested code blocks with varying backtick lengths

## [0.1.1] - 2025-XX-XX

### Added

- Library API for programmatic usage
- Extract mode with `--extract` and `--extract-path` flags
- Safe Markdown code fencing (dynamic backtick count)

## [0.1.0] - 2025-XX-XX

### Added

- Initial release
- Recursive directory scanning
- Memory-mapped (zero-copy) file reading
- Syntax highlighting based on file extension
- Binary file detection and placeholder output
- Custom ignore file support

[0.1.8]: https://github.com/MatiasHiltunen/src2md/compare/v0.1.7...v0.1.8
[0.1.7]: https://github.com/MatiasHiltunen/src2md/compare/v0.1.6...v0.1.7
[0.1.6]: https://github.com/MatiasHiltunen/src2md/compare/v0.1.5...v0.1.6
[0.1.5]: https://github.com/MatiasHiltunen/src2md/compare/v0.1.4...v0.1.5
[0.1.4]: https://github.com/MatiasHiltunen/src2md/compare/v0.1.1...v0.1.4
[0.1.1]: https://github.com/MatiasHiltunen/src2md/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/MatiasHiltunen/src2md/releases/tag/v0.1.0

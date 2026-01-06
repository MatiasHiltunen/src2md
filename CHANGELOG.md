# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/)

This project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
- Both `git` and `restore` features are now enabled by default
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

[0.1.5]: https://github.com/MatiasHiltunen/src2md/compare/v0.1.4...v0.1.5
[0.1.4]: https://github.com/MatiasHiltunen/src2md/compare/v0.1.1...v0.1.4
[0.1.1]: https://github.com/MatiasHiltunen/src2md/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/MatiasHiltunen/src2md/releases/tag/v0.1.0


# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.0.1] - 2025-10-16

### Fixed
- Updated the github actions release process

## [1.0.0] - 2025-10-16

### Added
- Initial Rust implementation of ExcelDiff
- Compare Excel (.xlsx) files worksheet by worksheet
- Color-coded diff output:
  - Modified cells with red text showing "old → new"
  - Yellow background for removed rows
  - Orange background for added rows
- Command-line interface with multiple options:
  - `--output, -o`: Specify output file path
  - `--sheet1`: Select sheet from first file
  - `--sheet2`: Select sheet from second file
  - `--diff-only`: Output only rows with differences
  - `--no-header`: Exclude header row when using --diff-only
  - `--ignore-whitespace`: Ignore whitespace differences in comparisons
- Extensible architecture with FileReader trait for supporting other formats
- Automatic column width adjustment in output files
- Support for various Excel data types (string, number, boolean, datetime)

### Performance
- Significantly faster than Python version
- Efficient memory usage
- No runtime dependencies (single static binary)

[Unreleased]: https://github.com/mumasoft/exceldiff/compare/v1.0.0...HEAD
[1.0.1]: https://github.com/mumasoft/exceldiff/releases/tag/v1.0.0...v1.0.1
[1.0.0]: https://github.com/mumasoft/exceldiff/releases/tag/v1.0.0

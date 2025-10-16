# ExcelDiff (Rust)

A Rust implementation of the ExcelDiff tool to compare two Excel worksheets and highlight differences with color-coded output.

## Features

- Compare Excel (.xlsx) files worksheet by worksheet
- Modified cells show both old and new values with red text and comments
- Color-coded diff output:
  - **Modified cells**: Red text showing "old → new" with explanatory comment
  - **Yellow rows**: Rows removed in the second file
  - **Orange rows**: Rows added in the second file
- Specify which worksheet to compare (defaults to first sheet)
- Option to output only rows with differences (exclude identical rows)
- Option to ignore whitespace differences (trims and collapses whitespace)
- Extensible architecture for adding support for other file formats

## Installation

### Option 1: Install from Source

```bash
# Clone the repository
git clone https://github.com/mumasoft/exceldiff
cd exceldiff

# Build and install
cargo install --path .
```

### Option 2: Build Locally

```bash
# Build in release mode
cargo build --release

# Binary will be at target/release/exceldiff
./target/release/exceldiff file1.xlsx file2.xlsx -o output.xlsx
```

## Usage

### Basic usage

Compare two Excel files and output to `diff_output.xlsx`:

```bash
exceldiff file1.xlsx file2.xlsx
```

### Specify output file

```bash
exceldiff file1.xlsx file2.xlsx --output result.xlsx
```

or using short form:

```bash
exceldiff file1.xlsx file2.xlsx -o result.xlsx
```

### Specify worksheets

Compare specific sheets within the files:

```bash
exceldiff file1.xlsx file2.xlsx --sheet1 "Sheet1" --sheet2 "Sheet1"
```

### Show only differences

Output only rows with differences (exclude identical rows):

```bash
exceldiff file1.xlsx file2.xlsx --diff-only
```

This is useful when comparing large files where you only want to see what changed.

By default, when using `--diff-only`, the first row from the first file is included as a header row. To exclude the header:

```bash
exceldiff file1.xlsx file2.xlsx --diff-only --no-header
```

### Ignore whitespace differences

Ignore differences in whitespace (leading/trailing spaces, tabs, newlines, etc.):

```bash
exceldiff file1.xlsx file2.xlsx --ignore-whitespace
```

This option:
- Trims leading and trailing whitespace from string values
- Collapses multiple consecutive whitespace characters (including newlines and tabs) into single spaces
- Useful when comparing files where formatting may differ but content is the same

Example: These would be considered identical with `--ignore-whitespace`:
- `"Hello  World"` and `"Hello World"`
- `"Test\nValue"` and `"Test Value"`
- `"  Data  "` and `"Data"`

### Full example

```bash
exceldiff baseline.xlsx updated.xlsx \
  --output comparison.xlsx \
  --sheet1 "Q1 Data" \
  --sheet2 "Q1 Data" \
  --diff-only \
  --ignore-whitespace
```

### Help

```bash
exceldiff --help
```

## Understanding the Output

The tool generates an Excel file with the following formatting:

### Modified Cells
For cells with different values, the output shows:
- Cell displays both values: `old_value → new_value` (separated by an arrow)
- **Red text color** to indicate the cell has changed
- **Cell comment** showing details: "Changed from: [old] To: [new]"

Example: If a cell changed from "25" to "26", it will display as "25 → 26" in red text, with a comment showing the change details

### Row Colors

| Color | Meaning |
|-------|---------|
| No color | Row is identical in both files |
| Yellow (entire row) | Row exists in file1 but not in file2 (removed) |
| Orange (entire row) | Row exists in file2 but not in file1 (added) |

## Architecture

The tool is designed with extensibility in mind:

```
src/
├── reader.rs         # FileReader trait and CellValue types
├── excel_reader.rs   # Excel implementation using calamine
├── differ.rs         # Core diff engine (format-agnostic)
├── writer.rs         # Excel output with formatting using rust_xlsxwriter
├── main.rs           # Command-line interface using clap
└── lib.rs            # Library exports
```

### Adding Support for Other Formats

To add support for CSV, ODS, or other formats:

1. Create a new reader struct implementing the `FileReader` trait in a new module
2. Implement the three required methods: `read()`, `get_sheet_names()`, and `supports()`
3. Update the CLI to use the appropriate reader based on file extension

Example:

```rust
use exceldiff::reader::{FileReader, Worksheet, Row, CellValue};
use anyhow::Result;

pub struct CSVReader;

impl FileReader for CSVReader {
    fn read(&self, file_path: &str, sheet_name: Option<&str>) -> Result<Worksheet> {
        // Implementation here
        todo!()
    }

    fn get_sheet_names(&self, file_path: &str) -> Result<Vec<String>> {
        Ok(vec!["Sheet1".to_string()])  // CSV has only one sheet
    }

    fn supports(&self, file_path: &str) -> bool {
        file_path.ends_with(".csv")
    }
}
```

## Dependencies

- **calamine** (0.26): For reading Excel files
- **rust_xlsxwriter** (0.82): For writing Excel files with formatting
- **clap** (4.5): For command-line argument parsing
- **anyhow** (1.0): For error handling
- **thiserror** (2.0): For custom error types

## Building for Release

To build an optimized release binary:

```bash
cargo build --release
```

The binary will be located at `target/release/exceldiff`.

### Cross-compilation

To build for different platforms:

```bash
# For Linux (x86_64)
cargo build --release --target x86_64-unknown-linux-gnu

# For macOS (ARM64)
cargo build --release --target aarch64-apple-darwin

# For Windows
cargo build --release --target x86_64-pc-windows-gnu
```

## Performance

The Rust version offers significant performance improvements over the Python version:
- Faster file reading and parsing
- More efficient memory usage
- Parallel processing capabilities (can be added)
- No runtime dependencies

## Releases

Pre-built binaries are automatically created for Linux and macOS when a version tag is pushed.

### Creating a Release

1. Update the version in `CHANGELOG.md` and document changes
2. Commit the changelog changes
3. Create and push a version tag:
   ```bash
   git tag v1.0.0
   git push origin v1.0.0
   ```

The GitHub Actions workflow will automatically:
- Build binaries for:
  - Linux x86_64
  - Linux ARM64 (aarch64)
  - macOS ARM64 (Apple Silicon)
  - macOS x86_64 (Intel)
- Extract release notes from CHANGELOG.md for the version
- Create a GitHub release with all binaries and SHA256 checksums
- Set the version in the binaries based on the git tag

### Version Information

- **Local builds**: Show version as "local"
- **Tagged releases**: Show the git tag version (e.g., "1.0.0")
- **CI builds**: Use the VERSION environment variable

Check the version:
```bash
exceldiff --version
```

### Available Downloads

Once released, binaries will be available at:
`https://github.com/mumasoft/exceldiff/releases`

Download and verify checksums:
```bash
# Download binary
wget https://github.com/mumasoft/exceldiff/releases/download/v1.0.0/exceldiff-linux-x86_64

# Download checksum
wget https://github.com/mumasoft/exceldiff/releases/download/v1.0.0/exceldiff-linux-x86_64.sha256

# Verify
shasum -a 256 -c exceldiff-linux-x86_64.sha256
```

## License

MIT

## Related Projects

- [ExcelDiff (Python)](https://github.com/mumasoft/exceldiff) - Original Python implementation

//! ExcelDiff - A tool to compare Excel worksheets and highlight differences
//!
//! This library provides functionality to compare two Excel files and generate
//! a color-coded diff output showing:
//! - Modified cells (red text with old → new values)
//! - Removed rows (yellow background)
//! - Added rows (orange background)

pub mod reader;
pub mod excel_reader;
pub mod differ;
pub mod writer;

pub use reader::FileReader;
pub use excel_reader::ExcelReader;
pub use differ::{DiffType, RowDiff, WorksheetDiffer};
pub use writer::ExcelDiffWriter;

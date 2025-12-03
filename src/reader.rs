//! File reader interface and implementations.

use anyhow::Result;

/// Cell value type that can hold various data types from Excel
#[derive(Debug, Clone, PartialEq)]
pub enum CellValue {
    String(String),
    Float(f64),
    Int(i64),
    Bool(bool),
    DateTime(f64), // Excel datetime stored as float (days since 1900-01-01)
    Empty,
}

impl CellValue {
    /// Normalize value for comparison (handles floating point precision)
    pub fn normalize(&self) -> Self {
        self.normalize_with_options(false)
    }

    /// Normalize value for comparison with options
    ///
    /// # Arguments
    /// * `ignore_whitespace` - If true, normalizes whitespace in strings (trims and collapses)
    pub fn normalize_with_options(&self, ignore_whitespace: bool) -> Self {
        match self {
            CellValue::Float(f) => CellValue::Float((f * 1e10).round() / 1e10),
            CellValue::DateTime(f) => CellValue::DateTime((f * 1e10).round() / 1e10),
            CellValue::String(s) if ignore_whitespace => {
                // Trim and collapse multiple whitespace characters (including newlines, tabs, etc.) into single spaces
                let normalized = s
                    .split_whitespace()
                    .collect::<Vec<&str>>()
                    .join(" ");
                CellValue::String(normalized)
            }
            other => other.clone(),
        }
    }

    /// Convert to display string
    pub fn to_string(&self) -> String {
        match self {
            CellValue::String(s) => s.clone(),
            CellValue::Float(f) => f.to_string(),
            CellValue::Int(i) => i.to_string(),
            CellValue::Bool(b) => b.to_string(),
            CellValue::DateTime(f) => f.to_string(), // Display as numeric value for comparison purposes
            CellValue::Empty => String::new(),
        }
    }
}

/// A row is a vector of cell values
pub type Row = Vec<CellValue>;

/// A worksheet is a vector of rows
pub type Worksheet = Vec<Row>;

/// Abstract trait for file readers that can read worksheet data
pub trait FileReader {
    /// Read a worksheet from a file
    ///
    /// # Arguments
    /// * `file_path` - Path to the file
    /// * `sheet_name` - Optional sheet name (None for first sheet)
    ///
    /// # Returns
    /// A worksheet containing rows of cell values
    fn read(&self, file_path: &str, sheet_name: Option<&str>) -> Result<Worksheet>;

    /// Get list of sheet names in the file
    ///
    /// # Arguments
    /// * `file_path` - Path to the file
    ///
    /// # Returns
    /// Vector of sheet names
    fn get_sheet_names(&self, file_path: &str) -> Result<Vec<String>>;

    /// Check if this reader supports the given file
    ///
    /// # Arguments
    /// * `file_path` - Path to the file
    ///
    /// # Returns
    /// true if the reader can handle this file
    fn supports(&self, file_path: &str) -> bool;
}

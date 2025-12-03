//! Excel file reader implementation using calamine.

use anyhow::{Context, Result};
use calamine::{open_workbook, Reader, Xlsx, Data};
use std::path::Path;

use crate::reader::{CellValue, FileReader, Row, Worksheet};

/// Reader for Excel (.xlsx) files
pub struct ExcelReader;

impl ExcelReader {
    pub fn new() -> Self {
        ExcelReader
    }
}

impl Default for ExcelReader {
    fn default() -> Self {
        Self::new()
    }
}

impl FileReader for ExcelReader {
    fn read(&self, file_path: &str, sheet_name: Option<&str>) -> Result<Worksheet> {
        if !self.supports(file_path) {
            anyhow::bail!("File {} is not a valid .xlsx file", file_path);
        }

        let mut workbook: Xlsx<_> = open_workbook(file_path)
            .with_context(|| format!("Failed to open workbook: {}", file_path))?;

        // Determine which sheet to read
        let sheet_to_read = if let Some(name) = sheet_name {
            name.to_string()
        } else {
            // Get first sheet name
            workbook
                .sheet_names()
                .first()
                .context("Workbook has no sheets")?
                .clone()
        };

        // Read the worksheet
        let range = workbook
            .worksheet_range(&sheet_to_read)
            .with_context(|| format!("Failed to read sheet: {}", sheet_to_read))?;

        // Convert range to our Worksheet type
        let mut worksheet = Worksheet::new();

        for row in range.rows() {
            let converted_row: Row = row
                .iter()
                .map(|cell| match cell {
                    Data::Int(i) => CellValue::Int(*i),
                    Data::Float(f) => CellValue::Float(*f),
                    Data::String(s) => CellValue::String(s.clone()),
                    Data::Bool(b) => CellValue::Bool(*b),
                    Data::Empty => CellValue::Empty,
                    Data::Error(_) => CellValue::Empty,
                    Data::DateTime(dt) => CellValue::DateTime(dt.as_f64()), // Store as DateTime to preserve formatting
                    Data::DateTimeIso(s) => CellValue::String(s.clone()),
                    Data::DurationIso(s) => CellValue::String(s.clone())
                })
                .collect();
            worksheet.push(converted_row);
        }

        Ok(worksheet)
    }

    fn get_sheet_names(&self, file_path: &str) -> Result<Vec<String>> {
        if !self.supports(file_path) {
            anyhow::bail!("File {} is not a valid .xlsx file", file_path);
        }

        let workbook: Xlsx<_> = open_workbook(file_path)
            .with_context(|| format!("Failed to open workbook: {}", file_path))?;

        Ok(workbook.sheet_names().to_vec())
    }

    fn supports(&self, file_path: &str) -> bool {
        Path::new(file_path)
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.eq_ignore_ascii_case("xlsx"))
            .unwrap_or(false)
    }
}

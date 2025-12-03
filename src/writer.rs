//! Excel writer with color formatting for diffs.

use anyhow::Result;
use rust_xlsxwriter::{Color, Format, Workbook};

use crate::differ::{DiffType, RowDiff};
use crate::reader::CellValue;

/// Writer for creating Excel files with diff highlighting
pub struct ExcelDiffWriter;

impl ExcelDiffWriter {
    pub fn new() -> Self {
        ExcelDiffWriter
    }

    /// Write diff results to an Excel file with color highlighting
    ///
    /// # Arguments
    /// * `diffs` - List of RowDiff objects
    /// * `output_path` - Path to write the output file
    /// * `diff_only` - If true, only write rows with differences (exclude identical rows)
    /// * `include_header` - If true, include the first row as header (only applies when diff_only=true)
    ///
    /// # Color scheme
    /// - Identical rows: No coloring
    /// - Modified rows: Red cells for changed values
    /// - Removed rows: Yellow background for entire row
    /// - Added rows: Orange background for entire row
    pub fn write(
        &self,
        diffs: &[RowDiff],
        output_path: &str,
        diff_only: bool,
        include_header: bool,
    ) -> Result<()> {
        let mut workbook = Workbook::new();
        let worksheet = workbook.add_worksheet();
        worksheet.set_name("Diff")?;

        // Create formats for different diff types
        let format_modified = Format::new().set_font_color(Color::Red);
        let format_removed = Format::new().set_background_color(Color::Yellow);
        let format_added = Format::new().set_background_color(Color::RGB(0xFFA500)); // Orange

        // Filter diffs if needed
        let diffs_to_write: Vec<&RowDiff> = if diff_only {
            diffs
                .iter()
                .filter(|d| d.diff_type != DiffType::Identical)
                .collect()
        } else {
            diffs.iter().collect()
        };

        // Include header row if requested
        let mut row_idx = 0u32;
        if include_header && !diffs.is_empty() {
            let header_row = &diffs[0];
            for (col_idx, value) in header_row.row_data.iter().enumerate() {
                self.write_cell(worksheet, row_idx, col_idx as u16, value, None)?;
            }
            row_idx += 1;
        }

        // Write all rows with appropriate formatting
        for diff in diffs_to_write {
            match diff.diff_type {
                DiffType::Identical => {
                    // No coloring for identical rows
                    for (col_idx, value) in diff.row_data.iter().enumerate() {
                        self.write_cell(worksheet, row_idx, col_idx as u16, value, None)?;
                    }
                }
                DiffType::Modified => {
                    // For modified cells, show both old and new values
                    for (col_idx, value) in diff.row_data.iter().enumerate() {
                        if diff.modified_cells.contains(&col_idx) {
                            if let Some(ref original_row) = diff.original_row_data {
                                let old_value = original_row.get(col_idx).unwrap_or(&CellValue::Empty);
                                let old_str = old_value.to_string();
                                let new_str = value.to_string();
                                let combined = format!("{} → {}", old_str, new_str);

                                // Write with red font
                                worksheet.write_string_with_format(
                                    row_idx,
                                    col_idx as u16,
                                    &combined,
                                    &format_modified,
                                )?;

                                // Note: Comments would be added here with worksheet.insert_note()
                                // but it requires a Note object which is more complex
                            } else {
                                self.write_cell(worksheet, row_idx, col_idx as u16, value, None)?;
                            }
                        } else {
                            // Cell not modified, just write the value
                            self.write_cell(worksheet, row_idx, col_idx as u16, value, None)?;
                        }
                    }
                }
                DiffType::Removed => {
                    // Color entire row yellow
                    for (col_idx, value) in diff.row_data.iter().enumerate() {
                        self.write_cell(
                            worksheet,
                            row_idx,
                            col_idx as u16,
                            value,
                            Some(&format_removed),
                        )?;
                    }
                }
                DiffType::Added => {
                    // Color entire row orange
                    for (col_idx, value) in diff.row_data.iter().enumerate() {
                        self.write_cell(
                            worksheet,
                            row_idx,
                            col_idx as u16,
                            value,
                            Some(&format_added),
                        )?;
                    }
                }
            }
            row_idx += 1;
        }

        // Auto-adjust column widths
        worksheet.autofit();

        workbook.save(output_path)?;
        Ok(())
    }

    /// Helper function to write a cell value with optional format
    fn write_cell(
        &self,
        worksheet: &mut rust_xlsxwriter::Worksheet,
        row: u32,
        col: u16,
        value: &CellValue,
        format: Option<&Format>,
    ) -> Result<()> {
        match value {
            CellValue::String(s) => {
                if let Some(fmt) = format {
                    worksheet.write_string_with_format(row, col, s, fmt)?;
                } else {
                    worksheet.write_string(row, col, s)?;
                }
            }
            CellValue::Int(i) => {
                if let Some(fmt) = format {
                    worksheet.write_number_with_format(row, col, *i as f64, fmt)?;
                } else {
                    worksheet.write_number(row, col, *i as f64)?;
                }
            }
            CellValue::Float(f) => {
                if let Some(fmt) = format {
                    worksheet.write_number_with_format(row, col, *f, fmt)?;
                } else {
                    worksheet.write_number(row, col, *f)?;
                }
            }
            CellValue::DateTime(dt) => {
                // Create a datetime format - use Excel's built-in datetime format
                let datetime_format = if let Some(fmt) = format {
                    // Clone and add datetime number format
                    fmt.clone().set_num_format("yyyy-mm-dd hh:mm:ss")
                } else {
                    // Create new format with datetime number format
                    Format::new().set_num_format("yyyy-mm-dd hh:mm:ss")
                };

                worksheet.write_number_with_format(row, col, *dt, &datetime_format)?;
            }
            CellValue::Bool(b) => {
                if let Some(fmt) = format {
                    worksheet.write_boolean_with_format(row, col, *b, fmt)?;
                } else {
                    worksheet.write_boolean(row, col, *b)?;
                }
            }
            CellValue::Empty => {
                if let Some(fmt) = format {
                    worksheet.write_string_with_format(row, col, "", fmt)?;
                } else {
                    worksheet.write_string(row, col, "")?;
                }
            }
        }
        Ok(())
    }
}

impl Default for ExcelDiffWriter {
    fn default() -> Self {
        Self::new()
    }
}

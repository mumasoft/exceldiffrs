//! Diff engine for comparing worksheets.

use std::collections::{HashMap, HashSet};
use crate::reader::{CellValue, Row, Worksheet};

/// Types of differences between rows
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DiffType {
    Identical,
    Modified,
    Removed,
    Added,
}

impl DiffType {
    pub fn as_str(&self) -> &'static str {
        match self {
            DiffType::Identical => "identical",
            DiffType::Modified => "modified",
            DiffType::Removed => "removed",
            DiffType::Added => "added",
        }
    }
}

/// Represents the diff information for a single row
#[derive(Debug, Clone)]
pub struct RowDiff {
    /// Index of the row in the result
    pub row_index: usize,
    /// Type of difference
    pub diff_type: DiffType,
    /// The actual row data (new values)
    pub row_data: Row,
    /// List of column indices that were modified (for Modified type)
    pub modified_cells: Vec<usize>,
    /// The original row data (old values, for Modified type)
    pub original_row_data: Option<Row>,
}

impl RowDiff {
    pub fn new(
        row_index: usize,
        diff_type: DiffType,
        row_data: Row,
        modified_cells: Vec<usize>,
        original_row_data: Option<Row>,
    ) -> Self {
        RowDiff {
            row_index,
            diff_type,
            row_data,
            modified_cells,
            original_row_data,
        }
    }
}

/// Engine for comparing two worksheets
pub struct WorksheetDiffer {
    ignore_whitespace: bool,
}

impl WorksheetDiffer {
    pub fn new() -> Self {
        WorksheetDiffer {
            ignore_whitespace: false,
        }
    }

    /// Create a new differ with options
    pub fn with_options(ignore_whitespace: bool) -> Self {
        WorksheetDiffer { ignore_whitespace }
    }

    /// Compare two worksheets and generate diff information
    pub fn compare(&self, sheet1: &Worksheet, sheet2: &Worksheet) -> Vec<RowDiff> {
        let mut result = Vec::new();

        // Normalize rows to handle different column counts
        let max_cols = sheet1
            .iter()
            .map(|r| r.len())
            .max()
            .unwrap_or(0)
            .max(sheet2.iter().map(|r| r.len()).max().unwrap_or(0));

        let sheet1_normalized: Vec<Row> = sheet1
            .iter()
            .map(|row| self.normalize_row(row, max_cols))
            .collect();
        let sheet2_normalized: Vec<Row> = sheet2
            .iter()
            .map(|row| self.normalize_row(row, max_cols))
            .collect();

        // Create mapping of rows for comparison
        let _sheet1_map: HashMap<Vec<u8>, usize> = sheet1_normalized
            .iter()
            .enumerate()
            .map(|(idx, row)| (self.row_to_key(row), idx))
            .collect();
        let sheet2_map: HashMap<Vec<u8>, usize> = sheet2_normalized
            .iter()
            .enumerate()
            .map(|(idx, row)| (self.row_to_key(row), idx))
            .collect();

        let mut processed_sheet1 = HashSet::new();
        let mut processed_sheet2 = HashSet::new();

        // First pass: find identical and modified rows
        for (idx1, row1) in sheet1_normalized.iter().enumerate() {
            let key1 = self.row_to_key(row1);

            if let Some(&idx2) = sheet2_map.get(&key1) {
                // Row exists in both sheets (identical)
                result.push(RowDiff::new(idx1, DiffType::Identical, row1.clone(), vec![], None));
                processed_sheet1.insert(idx1);
                processed_sheet2.insert(idx2);
            } else {
                // Check if this row has a modified version in sheet2
                if let Some((match_idx, modified_cells)) =
                    self.find_modified_row(row1, &sheet2_normalized, &processed_sheet2)
                {
                    // Found a modified version
                    result.push(RowDiff::new(
                        idx1,
                        DiffType::Modified,
                        sheet2_normalized[match_idx].clone(),
                        modified_cells,
                        Some(row1.clone()),
                    ));
                    processed_sheet1.insert(idx1);
                    processed_sheet2.insert(match_idx);
                } else {
                    // Row removed in sheet2
                    result.push(RowDiff::new(idx1, DiffType::Removed, row1.clone(), vec![], None));
                    processed_sheet1.insert(idx1);
                }
            }
        }

        // Second pass: find added rows (in sheet2 but not in sheet1)
        for (idx2, row2) in sheet2_normalized.iter().enumerate() {
            if !processed_sheet2.contains(&idx2) {
                result.push(RowDiff::new(
                    result.len(),
                    DiffType::Added,
                    row2.clone(),
                    vec![],
                    None,
                ));
            }
        }

        result
    }

    /// Normalize a row to a target length by padding with Empty
    fn normalize_row(&self, row: &Row, target_length: usize) -> Row {
        if row.len() >= target_length {
            row[..target_length].to_vec()
        } else {
            let mut normalized = row.clone();
            normalized.resize(target_length, CellValue::Empty);
            normalized
        }
    }

    /// Convert a row to a hashable key for comparison
    fn row_to_key(&self, row: &Row) -> Vec<u8> {
        // Create a simple hash by concatenating normalized string representations
        let mut key = Vec::new();
        for value in row {
            let normalized = value.normalize_with_options(self.ignore_whitespace);
            let s = normalized.to_string();
            key.extend_from_slice(s.as_bytes());
            key.push(0); // separator
        }
        key
    }

    /// Find a row that matches the target row with some modifications
    ///
    /// Returns (row_index, list of modified cell indices) or None
    fn find_modified_row(
        &self,
        target_row: &Row,
        sheet: &[Row],
        processed: &HashSet<usize>,
    ) -> Option<(usize, Vec<usize>)> {
        // Simple heuristic: if more than 50% of cells match, consider it a modified row
        let mut best_match: Option<usize> = None;
        let mut best_score = 0.0;
        let mut best_modified = Vec::new();

        for (idx, row) in sheet.iter().enumerate() {
            if processed.contains(&idx) {
                continue;
            }

            let mut matches = 0;
            let mut modified = Vec::new();

            for (col_idx, (v1, v2)) in target_row.iter().zip(row.iter()).enumerate() {
                if v1.normalize_with_options(self.ignore_whitespace)
                    == v2.normalize_with_options(self.ignore_whitespace)
                {
                    matches += 1;
                } else {
                    modified.push(col_idx);
                }
            }

            let score = if !target_row.is_empty() {
                matches as f64 / target_row.len() as f64
            } else {
                0.0
            };

            // Require at least 50% match to consider it a modification
            if score > best_score && score >= 0.5 {
                best_score = score;
                best_match = Some(idx);
                best_modified = modified;
            }
        }

        best_match.map(|idx| (idx, best_modified))
    }
}

impl Default for WorksheetDiffer {
    fn default() -> Self {
        Self::new()
    }
}

//! Defines the core data structures for cell values and error handling in the spreadsheet.
//!
//! This module provides the fundamental types needed to represent cell data,
//! including various value types, error conditions, and formula storage.

use crate::common::expression::Expression;
use serde::{Deserialize, Serialize};

/// Represents possible error conditions that can occur during cell evaluation.
///
/// These errors are used to track and propagate problems encountered when
/// calculating cell values based on formulas.
#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum CellError {
    /// Error that occurs when attempting to divide by zero.
    DivideByZero,
    /// Error that occurs when a numeric operation depends on a non-numeric value.
    DependsOnNonNumeric,
    /// Error that occurs when a cell depends on another cell containing an error.
    DependsOnErr,
}

/// Represents the possible values a cell can contain.
///
/// Cells can be empty, contain string data, or contain numeric data.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub enum CellValue {
    /// Represents an empty cell with no value.
    #[default]
    Empty,
    /// Contains a string value.
    String(String),
    /// Contains a numeric value.
    Number(f64),
}

/// Represents the complete data for a cell, including its value and formula.
///
/// A cell can contain a computed value (or error) and optionally a formula
/// that was used to calculate that value.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CellData {
    /// The evaluated value of the cell, or an error if evaluation failed.
    pub value: Result<CellValue, CellError>,
    /// The formula expression for this cell, if any.
    pub formula: Option<Expression>,
}

impl Default for CellData {
    /// Creates a default cell data instance with empty value and no formula.
    fn default() -> Self {
        CellData {
            value: Ok(CellValue::Empty),
            formula: None,
        }
    }
}

impl CellData {
    /// Returns a static reference to the default cell data instance.
    ///
    /// This method is useful for cases where a reference to a default cell is needed
    /// without creating a new instance each time.
    pub fn default_instance() -> &'static CellData {
        static DEFAULT_CELL: CellData = CellData {
            value: Ok(CellValue::Empty),
            formula: None,
        };
        &DEFAULT_CELL
    }
}

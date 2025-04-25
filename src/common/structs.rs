//! This module contains the implementation of the common grammar of the spreadsheet -
//! The cells are of two types - Absolute and Relative.
//!
//! The Relative and Absolute Cells are different structs to allow easy
//! implementation of relative formula copying, a feature much more useful than absolute formula
//! copying and the default in popular spreadsheet software like Excel.
//!
//! All the formulas in the spreadsheet are stored as relative cells, and then converted to absolute cells
//! at the time of evaluation.
//!
//! These being different structs makes a conversion mistake impossible, as the structs are not interchangeable.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::fmt::Display;
use std::str::FromStr;

#[derive(
    Debug, Clone, PartialEq, Eq, Default, Hash, PartialOrd, Ord, Copy, Serialize, Deserialize,
)]
pub struct AbsCell {
    pub row: i16,
    pub col: i16,
}

#[derive(
    Debug, Clone, PartialEq, Eq, Default, Hash, PartialOrd, Ord, Copy, Serialize, Deserialize,
)]
pub struct RelCell {
    pub row: i16,
    pub col: i16,
}

/// Represents a cell in a spreadsheet using absolute coordinates.
/// Stored as 0-indexed, but displayed as 1-indexed.
/// Prefer using `AbsCell::FromStr` to create an `AbsCell` from a string instead of manually creating the instance.
impl AbsCell {
    pub const fn new(row: i16, col: i16) -> Self {
        AbsCell { row, col }
    }

    /// Creates a new `AbsCell` from a `RelCell` and an `AbsCell` origin.
    /// This is useful for converting relative cell references to absolute ones
    /// during evaluation.
    pub fn from_rel(target: RelCell, origin: AbsCell) -> Self {
        AbsCell {
            row: origin.row + target.row,
            col: origin.col + target.col,
        }
    }

    pub fn from_rel_origin(target: RelCell) -> Self {
        AbsCell {
            row: target.row,
            col: target.col,
        }
    }

    /// Converts an `AbsCell` to a `RelCell` using the given origin.
    /// This is useful for converting absolute cell references to relative ones
    /// during parsing the formula.
    pub fn to_rel(&self, origin: AbsCell) -> RelCell {
        RelCell {
            row: self.row - origin.row,
            col: self.col - origin.col,
        }
    }
}

impl Display for AbsCell {
    /// Converts the `AbsCell` to a string representation in spreadsheet format (e.g., "A1", "B2").
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Convert col number (0-indexed) into letters
        let mut col = self.col as usize;
        let mut col_str = String::new();
        col += 1; // make it 1-indexed for spreadsheet-style lettering

        while col > 0 {
            let rem = (col - 1) % 26;
            col_str.insert(0, (b'A' + rem as u8) as char);
            col = (col - 1) / 26;
        }

        // Row is 0-indexed in struct, but spreadsheet rows start at 1
        write!(f, "{}{}", col_str, self.row + 1)
    }
}

impl FromStr for AbsCell {
    type Err = String;

    /// Parses a string representation of a cell (e.g., "A1", "B2") into an `AbsCell`.
    /// The interpretation is 0 based, so "A1" is (0, 0) and "B2" is (1, 1).
    /// Returns an error if the string is not a valid cell reference.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut col = 0i16;
        let mut row_part = String::new();

        for (i, c) in s.chars().enumerate() {
            if c.is_ascii_alphabetic() {
                let upper_c = c.to_ascii_uppercase();
                if !upper_c.is_ascii_uppercase() {
                    return Err(format!("Invalid column letter: {}", c));
                }
                col = col * 26 + ((upper_c as u8 - b'A') as i16 + 1);
            } else if c.is_ascii_digit() {
                row_part = s[i..].to_string();
                break;
            } else {
                return Err(format!("Invalid character in cell: {}", c));
            }
        }

        if row_part.is_empty() {
            return Err("Missing row number".to_string());
        }

        let row: i16 = row_part.parse().map_err(|_| "Invalid row number")?;

        Ok(AbsCell {
            col: col - 1, // back to 0-indexed
            row: row - 1, // back to 0-indexed
        })
    }
}

impl RelCell {
    pub fn new(x: i16, y: i16) -> Self {
        RelCell { row: x, col: y }
    }

    pub fn to_abs(&self, origin: AbsCell) -> AbsCell {
        AbsCell {
            row: origin.row + self.row,
            col: origin.col + self.col,
        }
    }
}

use std::fmt;
use std::fmt::Display;
use std::str::FromStr;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Default, Hash, PartialOrd, Ord, Copy, Serialize, Deserialize)] 
pub struct AbsCell {
    pub row: i16,
    pub col: i16,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Hash, PartialOrd, Ord, Copy, Serialize, Deserialize)]
pub struct RelCell {
    pub row: i16,
    pub col: i16,
}

impl AbsCell {
     pub const fn new(row: i16, col: i16) -> Self {
        
        AbsCell { row, col }
    }
    
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
    
    pub fn to_rel(&self, origin: AbsCell) -> RelCell {
        RelCell {
            row: self.row - origin.row,
            col: self.col - origin.col,
        }
    }
}

impl Display for AbsCell {
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

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut col = 0i16;
        let mut row_part = String::new();
        
        for (i, c) in s.chars().enumerate() {
            if c.is_ascii_alphabetic() {
                let upper_c = c.to_ascii_uppercase();
                if !(b'A'..=b'Z').contains(&(upper_c as u8)) {
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
//! A parser for handling spreadsheet-related expressions.
//!
//! The `MyParser` struct provides methods to parse and interpret common expressions
//! encountered in spreadsheet applications. This includes converting cell names to coordinates,
//! splitting binary expressions, and parsing range-based function calls.
pub struct MyParser;

/// Represents a range in a spreadsheet as a tuple.
///
/// The tuple contains:
/// - The function name as a string slice (e.g., "SUM", "MAX").
/// - The starting cell's coordinates as a tuple `(col, row)`.
/// - The ending cell's coordinates as a tuple `(col, row)`.
type RangeType<'a> = (&'a str, (u16, u16), (u16, u16));

impl MyParser {
    /// Converts a cell name (e.g., "A1") into its column and row indices.
    ///
    /// This method processes a string representing a cell name in a spreadsheet
    /// (e.g., "A1", "Z10") and converts it into numerical indices for the column and row.
    /// Columns are represented as numbers based on their alphabetical order
    /// (e.g., "A" = 1, "B" = 2, ..., "Z" = 26, "AA" = 27).
    ///
    /// # Arguments
    /// * `s` - A string slice representing the cell name.
    ///
    /// # Returns
    /// * `Some((col, row))` - The column and row indices if parsing succeeds.
    /// * `None` - If the input string is invalid (e.g., lacks letters or numbers).
    ///
    /// # Examples
    /// ```rust
    /// use embedded::myparser::MyParser;
    /// assert_eq!(MyParser::cell_name_to_coord("A1"), Some((1, 1)));
    /// assert_eq!(MyParser::cell_name_to_coord("Z10"), Some((26, 10)));
    /// assert_eq!(MyParser::cell_name_to_coord("AA15"), Some((27, 15)));
    /// assert_eq!(MyParser::cell_name_to_coord("1A"), None); // Invalid input
    /// ```
    pub fn cell_name_to_coord(s: &str) -> Option<(u16, u16)> {
        let trimmed = s.trim();
        let mut chars = trimmed.chars();
        if let Some(first) = chars.next() {
            if !first.is_alphabetic() {
                return None;
            }
        } else {
            return None;
        }

        let letters: String = trimmed.chars().take_while(|c| c.is_alphabetic()).collect();
        let numbers: String = trimmed.chars().skip_while(|c| c.is_alphabetic()).collect();

        if letters.is_empty() || numbers.is_empty() {
            return None;
        }

        let col = letters.chars().fold(0, |acc, c| {
            acc * 26 + ((c.to_ascii_uppercase() as u16) - ('A' as u16) + 1)
        });
        let row = numbers.parse::<u16>().ok()?;

        Some((col, row))
    }

    /// Splits a binary expression into its operator and operands.
    ///
    /// This method parses simple binary expressions containing one of the supported operators
    /// (`+`, `-`, `*`, `/`) and returns the operator, the left-hand side (LHS), and the right-hand
    /// side (RHS) as string slices.
    ///
    /// # Arguments
    /// * `expr` - A string slice representing the binary expression.
    ///
    /// # Returns
    /// * `Some((op, lhs, rhs))` - A tuple containing:
    ///   - `op` - The operator as a character.
    ///   - `lhs` - The left-hand operand as a string slice.
    ///   - `rhs` - The right-hand operand as a string slice.
    /// * `None` - If the expression is invalid or cannot be parsed.
    ///
    /// # Supported Operators
    /// - `+`, `-`, `*`, `/`
    ///
    /// # Examples
    /// ```rust
    /// use embedded::myparser::MyParser;
    /// assert_eq!(MyParser::split_binary("B1+C2"), Some(('+', "B1", "C2")));
    /// assert_eq!(MyParser::split_binary("5*3"), Some(('*', "5", "3")));
    /// assert_eq!(MyParser::split_binary("A1/2"), Some(('/', "A1", "2")));
    /// assert_eq!(MyParser::split_binary("A1+"), None); // Missing RHS
    /// assert_eq!(MyParser::split_binary("+A1"), None); // Missing LHS
    /// ```
    pub fn split_binary(expr: &str) -> Option<(char, &str, &str)> {
        for op in ['+', '-', '*', '/'] {
            if let Some(idx) = expr.find(op) {
                let (lhs, rest) = expr.split_at(idx);
                let rhs = &rest[1..];
                if !lhs.trim().is_empty() && !rhs.trim().is_empty() {
                    return Some((op, lhs, rhs));
                }
            }
        }
        None
    }

    /// Parses a range-based function call (e.g., `MAX(A1:B3)`).
    ///
    /// This method interprets functions that operate over a range of cells, such as `SUM`, `MAX`,
    /// and `AVG`. It identifies the function name, the starting cell, and the ending cell.
    ///
    /// # Arguments
    /// * `expr` - A string slice representing the function expression.
    ///
    /// # Returns
    /// * `Some((func, start, end))` - A tuple containing:
    ///   - `func` - The function name as a string slice (e.g., "MAX").
    ///   - `start` - The starting cell's coordinates as `(col, row)`.
    ///   - `end` - The ending cell's coordinates as `(col, row)`.
    /// * `None` - If the expression is invalid or cannot be parsed.
    ///
    /// # Supported Functions
    /// - `MIN`, `MAX`, `AVG`, `SUM`, `STDEV`, `SLEEP`
    ///
    /// # Examples
    /// ```rust
    /// use embedded::myparser::MyParser;
    /// assert_eq!(
    ///     MyParser::parse_range("MAX(A1:B3)"),
    ///     Some(("MAX", (1, 1), (2, 3)))
    /// );
    /// assert_eq!(
    ///     MyParser::parse_range("SUM(Z10:AA20)"),
    ///     Some(("SUM", (26, 10), (27, 20)))
    /// );
    /// assert_eq!(MyParser::parse_range("INVALID_FUNC(A1:B3)"), None); // Unsupported function
    /// assert_eq!(MyParser::parse_range("MAX(A1)"), None); // Missing range
    /// ```
    pub fn parse_range(expr: &str) -> Option<RangeType> {
        let expr = expr.trim();
        for &func in &["MIN", "MAX", "AVG", "SUM", "STDEV", "SLEEP"] {
            let open = format!("{}(", func);
            if expr.starts_with(&open) && expr.ends_with(')') {
                let inside = &expr[open.len()..expr.len() - 1];
                if let Some(colon) = inside.find(':') {
                    let a = inside[..colon].trim();
                    let b = inside[colon + 1..].trim();
                    if let (Some(s), Some(e)) = (
                        MyParser::cell_name_to_coord(a),
                        MyParser::cell_name_to_coord(b),
                    ) {
                        return Some((func, s, e));
                    }
                }
            }
        }
        None
    }
}

use crate::myparser::MyParser;
use crate::spreadsheet::Spreadsheet;
use std::io::{self, BufRead, Write};
use std::time::Instant;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum CommandResult {
    Ok,
    InvalidCell,
    InvalidRange,
    UnrecognizedCommand,
    CircularDependency,
    DivisionByZero,
    Quit,
}

impl CommandResult {
    pub fn from_code(code: u8) -> Self {
        match code {
            0 => CommandResult::Ok,
            1 => CommandResult::InvalidCell,
            2 => CommandResult::InvalidRange,
            3 => CommandResult::UnrecognizedCommand,
            4 => CommandResult::CircularDependency,
            5 => CommandResult::DivisionByZero,
            _ => CommandResult::UnrecognizedCommand,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            CommandResult::Ok => "ok",
            CommandResult::InvalidCell => "Invalid cell",
            CommandResult::InvalidRange => "Invalid range",
            CommandResult::UnrecognizedCommand => "unrecognized cmd",
            CommandResult::CircularDependency => "Circular dependency",
            CommandResult::DivisionByZero => "Division_by_zero",
            CommandResult::Quit => "quit",
        }
    }
}

pub struct CommandHandler {
    viewport_row: usize,
    viewport_col: usize,
    output_enabled: bool,
    last_result: CommandResult,
    last_instant: Instant,
}

impl CommandHandler {
    pub fn new() -> Self {
        CommandHandler {
            viewport_row: 0,
            viewport_col: 0,
            output_enabled: true,
            last_result: CommandResult::Ok,
            last_instant: Instant::now(),
        }
    }

    pub fn handle_command(&mut self, command: &str, sheet: &mut Spreadsheet) -> CommandResult {
        self.last_instant = Instant::now();

        let command = command.trim_end();

        if command.is_empty() {
            return CommandResult::Ok;
        }

        let result = if command == "q" || command == "Q" {
            CommandResult::Quit
        } else if command == "disable_output" {
            self.output_enabled = false;
            CommandResult::Ok
        } else if command == "enable_output" {
            self.output_enabled = true;
            CommandResult::Ok
        } else if command.starts_with("scroll_to") {
            self.handle_scroll_to(command, sheet)
        } else if command == "w" {
            self.viewport_row = self.viewport_row.saturating_sub(10);
            CommandResult::Ok
        } else if command == "s" {
            if sheet.rows <= 10 {
                self.viewport_row = 0;
            } else if self.viewport_row + 20 < sheet.rows {
                self.viewport_row += 10;
            } else {
                self.viewport_row = sheet.rows - 10;
            }
            CommandResult::Ok
        } else if command == "a" {
            self.viewport_col = self.viewport_col.saturating_sub(10);
            CommandResult::Ok
        } else if command == "d" {
            if sheet.cols <= 10 {
                self.viewport_col = 0;
            } else if self.viewport_col + 20 < sheet.cols {
                self.viewport_col += 10;
            } else {
                self.viewport_col = sheet.cols - 10;
            }
            CommandResult::Ok
        } else if let Some(pos) = command.find('=') {
            self.handle_cell_assignment(command, pos, sheet)
        } else {
            CommandResult::UnrecognizedCommand
        };

        self.last_result = result;
        result
    }

    fn handle_scroll_to(&mut self, command: &str, sheet: &Spreadsheet) -> CommandResult {
        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.len() >= 2 {
            // Use the parser for label conversion as in the original code
            if let Some((col, row)) = MyParser::cell_name_to_coord(parts[1]) {
                // Convert 1-indexed to zero-indexed (cell coords are 1-indexed, viewport is 0-indexed)
                let row_idx = row as usize - 1;
                let col_idx = col as usize - 1;
                if row_idx < sheet.rows && col_idx < sheet.cols {
                    self.viewport_row = row_idx;
                    self.viewport_col = col_idx;
                    return CommandResult::Ok;
                }
            }
        }
        CommandResult::InvalidCell
    }

    fn handle_cell_assignment(
        &mut self,
        command: &str,
        pos: usize,
        sheet: &mut Spreadsheet,
    ) -> CommandResult {
        let (cell_str, expr) = command.split_at(pos);
        let expr = &expr[1..]; // skip '='

        if let Some((col, row)) = MyParser::cell_name_to_coord(cell_str.trim()) {
            let result_code = sheet.set_cell((col, row), expr);
            CommandResult::from_code(result_code)
        } else {
            CommandResult::InvalidCell
        }
    }

    // Display prompt with elapsed time and status message, matching the original format exactly
    pub fn display_prompt(&self, writer: &mut impl Write) -> io::Result<()> {
        let elapsed = self.last_instant.elapsed().as_secs_f64();
        write!(
            writer,
            "[{:.1}] ({}) > ",
            elapsed,
            self.last_result.as_str()
        )?;
        writer.flush()
    }

    pub fn should_display(&self) -> bool {
        self.output_enabled
    }

    pub fn get_viewport(&self) -> (usize, usize) {
        (self.viewport_row, self.viewport_col)
    }
}

/// Handles user commands for interacting with the spreadsheet.
///
/// This function provides an interactive terminal-based interface, allowing users
/// to manipulate and navigate a spreadsheet. It supports commands for navigation,
/// modifying cells, enabling/disabling output, and more.
///
/// # Arguments
/// * `sheet` - A mutable reference to the `Spreadsheet` object.
///
/// # Commands
/// - `q` or `Q`: Quit the command interface.
/// - `disable_output`: Disable spreadsheet display updates.
/// - `enable_output`: Enable spreadsheet display updates.
/// - `scroll_to <cell>`: Scroll to a specific cell (e.g., `scroll_to A1`).
/// - `w`, `a`, `s`, `d`: Navigate the spreadsheet's viewport (up, left, down, right).
/// - `<cell>=<expression>`: Set a cell's value or formula (e.g., `A1=5+3`).
///
/// # Behavior
/// - Displays the spreadsheet's current state in a 10x10 viewport.
/// - Handles viewport boundaries and ensures safe scrolling.
/// - Provides status messages for the last command's result (e.g., "ok", "Invalid cell").
///
/// # Notes
/// - Errors such as invalid cells, circular dependencies, or division by zero are reported
///   via status messages.
/// - The viewport and cell coordinates are zero-indexed internally but support user-friendly
///   one-indexed labels through `MyParser`.
///
/// # Example
/// ```rust
/// use embedded::commands::handle_commands;
/// use crate::embedded::myparser::MyParser;
/// use crate::embedded::spreadsheet::Spreadsheet;
///
/// let mut sheet = Spreadsheet::new(20, 20);
/// handle_commands(&mut sheet);
/// ```
pub fn handle_commands(sheet: &mut Spreadsheet) {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut command_handler = CommandHandler::new();
    let mut input = String::new();

    // Initial display
    sheet.display(0, 0, 10, 10);

    loop {
        command_handler.display_prompt(&mut stdout).unwrap();

        input.clear();
        if stdin.lock().read_line(&mut input).unwrap() == 0 {
            break;
        }

        let result = command_handler.handle_command(&input, sheet);

        if let CommandResult::Quit = result {
            break;
        }

        if command_handler.should_display() {
            let (viewport_row, viewport_col) = command_handler.get_viewport();
            sheet.display(viewport_row, viewport_col, 10, 10);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spreadsheet::Spreadsheet;

    #[test]
    fn test_command_result_from_code() {
        assert!(matches!(CommandResult::from_code(0), CommandResult::Ok));
        assert!(matches!(
            CommandResult::from_code(1),
            CommandResult::InvalidCell
        ));
        assert!(matches!(
            CommandResult::from_code(2),
            CommandResult::InvalidRange
        ));
        assert!(matches!(
            CommandResult::from_code(3),
            CommandResult::UnrecognizedCommand
        ));
        assert!(matches!(
            CommandResult::from_code(4),
            CommandResult::CircularDependency
        ));
        assert!(matches!(
            CommandResult::from_code(5),
            CommandResult::DivisionByZero
        ));
        assert!(matches!(
            CommandResult::from_code(6),
            CommandResult::UnrecognizedCommand
        )); // Unknown codes default to unrecognized
    }

    #[test]
    fn test_command_result_as_str() {
        assert_eq!(CommandResult::Ok.as_str(), "ok");
        assert_eq!(CommandResult::InvalidCell.as_str(), "Invalid cell");
        assert_eq!(CommandResult::InvalidRange.as_str(), "Invalid range");
        assert_eq!(
            CommandResult::UnrecognizedCommand.as_str(),
            "unrecognized cmd"
        );
        assert_eq!(
            CommandResult::CircularDependency.as_str(),
            "Circular dependency"
        );
        assert_eq!(CommandResult::DivisionByZero.as_str(), "Division_by_zero");
        assert_eq!(CommandResult::Quit.as_str(), "quit");
    }

    #[test]
    fn test_new_command_handler() {
        let handler = CommandHandler::new();
        assert_eq!(handler.viewport_row, 0);
        assert_eq!(handler.viewport_col, 0);
        assert!(handler.output_enabled);
        assert!(matches!(handler.last_result, CommandResult::Ok));
    }

    #[test]
    fn test_quit_commands() {
        let mut handler = CommandHandler::new();
        let mut sheet = Spreadsheet::new(10, 10);

        assert!(matches!(
            handler.handle_command("q", &mut sheet),
            CommandResult::Quit
        ));
        assert!(matches!(
            handler.handle_command("Q", &mut sheet),
            CommandResult::Quit
        ));
    }

    #[test]
    fn test_cell_assignment() {
        let mut handler = CommandHandler::new();
        let mut sheet = Spreadsheet::new(10, 10);

        let result = handler.handle_command("", &mut sheet);
        assert!(matches!(result, CommandResult::Ok));

        // Valid cell assignment
        let result = handler.handle_command("A1=42", &mut sheet);
        assert!(matches!(result, CommandResult::Ok));

        // Invalid cell reference
        let result = handler.handle_command("Z99=42", &mut sheet);
        assert!(matches!(result, CommandResult::InvalidCell));

        // Invalid formula
        let result = handler.handle_command("A2=invalid", &mut sheet);
        assert!(matches!(result, CommandResult::UnrecognizedCommand));

        // Formula that references other cells
        let _ = handler.handle_command("A2=A1", &mut sheet);

        // Circular reference
        let result = handler.handle_command("A1=A2", &mut sheet);
        assert!(matches!(result, CommandResult::CircularDependency));
    }

    #[test]
    fn test_output_toggle_commands() {
        let mut handler = CommandHandler::new();
        let mut sheet = Spreadsheet::new(10, 10);

        // Initially output is enabled
        assert!(handler.should_display());

        // Disable output
        let result = handler.handle_command("disable_output", &mut sheet);
        assert!(matches!(result, CommandResult::Ok));
        assert!(!handler.should_display());

        // Enable output
        let result = handler.handle_command("enable_output", &mut sheet);
        assert!(matches!(result, CommandResult::Ok));
        assert!(handler.should_display());
    }

    #[test]
    fn test_scroll_to_command() {
        let mut handler = CommandHandler::new();
        let mut sheet = Spreadsheet::new(20, 20);

        // Valid scroll
        let result = handler.handle_command("scroll_to B3", &mut sheet);
        assert!(matches!(result, CommandResult::Ok));
        assert_eq!(handler.get_viewport(), (2, 1)); // 1-indexed to 0-indexed

        // Invalid cell reference
        let result = handler.handle_command("scroll_to Z99", &mut sheet);
        assert!(matches!(result, CommandResult::InvalidCell));

        // Invalid command format
        let result = handler.handle_command("scroll_to", &mut sheet);
        assert!(matches!(result, CommandResult::InvalidCell));

        // Verify viewport didn't change
        assert_eq!(handler.get_viewport(), (2, 1));
    }

    #[test]
    fn test_navigation_commands() {
        let mut handler = CommandHandler::new();
        let mut sheet = Spreadsheet::new(30, 30);

        // Set initial position to row 15, col 15
        handler.viewport_row = 15;
        handler.viewport_col = 15;

        // Test 'w' (up)
        let result = handler.handle_command("w", &mut sheet);
        assert!(matches!(result, CommandResult::Ok));
        assert_eq!(handler.get_viewport(), (5, 15)); // Moved up 10 rows

        // Test 's' (down)
        let result = handler.handle_command("s", &mut sheet);
        assert!(matches!(result, CommandResult::Ok));
        assert_eq!(handler.get_viewport(), (15, 15)); // Moved down 10 rows

        // Test 'a' (left)
        let result = handler.handle_command("a", &mut sheet);
        assert!(matches!(result, CommandResult::Ok));
        assert_eq!(handler.get_viewport(), (15, 5)); // Moved left 10 columns

        // Test 'd' (right)
        let result = handler.handle_command("d", &mut sheet);
        assert!(matches!(result, CommandResult::Ok));
        assert_eq!(handler.get_viewport(), (15, 15)); // Moved right 10 columns

        // Test edge case: can't go above row 0
        handler.viewport_row = 5;
        let result = handler.handle_command("w", &mut sheet);
        assert!(matches!(result, CommandResult::Ok));
        assert_eq!(handler.get_viewport(), (0, 15)); // Clamped to 0

        // Test edge case: can't go to the left of column 0
        handler.viewport_col = 5;
        let result = handler.handle_command("a", &mut sheet);
        assert!(matches!(result, CommandResult::Ok));
        assert_eq!(handler.get_viewport(), (0, 0)); // Clamped to 0

        // Test edge case: going beyond bottom of sheet (matches original behavior)
        handler.viewport_row = sheet.rows - 15;
        let result = handler.handle_command("s", &mut sheet);
        assert!(matches!(result, CommandResult::Ok));
        assert_eq!(handler.get_viewport(), (sheet.rows - 10, 0));

        // Test edge case: going beyond right edge of sheet (matches original behavior)
        handler.viewport_col = sheet.cols - 15;
        let result = handler.handle_command("d", &mut sheet);
        assert!(matches!(result, CommandResult::Ok));
        assert_eq!(handler.get_viewport(), (sheet.rows - 10, sheet.cols - 10));

        // Test exact sized sheet navigation (10x10)
        let mut exact_sheet = Spreadsheet::new(10, 10);
        handler.viewport_row = 0;
        handler.viewport_col = 0;

        let result = handler.handle_command("s", &mut exact_sheet);
        assert!(matches!(result, CommandResult::Ok));
        assert_eq!(handler.get_viewport(), (0, 0)); // Can't scroll down in 10x10 sheet

        let result = handler.handle_command("d", &mut exact_sheet);
        assert!(matches!(result, CommandResult::Ok));
        assert_eq!(handler.get_viewport(), (0, 0)); // Can't scroll right in 10x10 sheet

        // Test small sheet navigation (smaller than viewport)
        let mut small_sheet = Spreadsheet::new(8, 8);
        handler.viewport_row = 0;
        handler.viewport_col = 0;

        let result = handler.handle_command("s", &mut small_sheet);
        assert!(matches!(result, CommandResult::Ok));
        assert_eq!(handler.get_viewport(), (0, 0)); // Can't scroll down in small sheet

        let result = handler.handle_command("d", &mut small_sheet);
        assert!(matches!(result, CommandResult::Ok));
        assert_eq!(handler.get_viewport(), (0, 0)); // Can't scroll right in small sheet
    }

    #[test]
    fn test_unrecognized_command() {
        let mut handler = CommandHandler::new();
        let mut sheet = Spreadsheet::new(10, 10);

        let result = handler.handle_command("unknown_command", &mut sheet);
        assert!(matches!(result, CommandResult::UnrecognizedCommand));
    }

    #[test]
    fn test_display_prompt() {
        let handler = CommandHandler::new();
        let mut output = Vec::new();

        handler.display_prompt(&mut output).unwrap();

        // The exact content will vary because of the elapsed time, but it should contain the status
        let output_str = String::from_utf8(output).unwrap();
        assert!(output_str.contains("ok"));
        assert!(output_str.contains(">"));
    }
}

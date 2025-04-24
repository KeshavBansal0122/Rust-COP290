use std::env;
mod commands;
mod function;
mod myparser;
mod spreadsheet;
use embedded::ui;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args();
    let _exe = args.next(); // skip binary name

    match args.next().as_deref() {
        // If first arg is exactly "ext1", launch the GUI:
        Some("ext1") => {
            ui::run_spreadsheet_app()?;
        }

        // Otherwise expect two numeric args: rows and cols
        Some(rows_str) => {
            let cols_str = args.next()
                .ok_or_else(|| "Usage: <ext1> | <rows> <cols>")?;
            let rows: usize = rows_str.parse()?;
            let cols: usize = cols_str.parse()?;

            if rows == 0 || rows > 999 || cols == 0 || cols > 18_278 {
                eprintln!(
                    "Error: Invalid rows or cols; got {}×{}. Valid: 1≤rows≤999, 1≤cols≤18278.",
                    rows, cols
                );
                std::process::exit(1);
            }
            let mut sheet = spreadsheet::Spreadsheet::new(rows, cols);
            commands::handle_commands(&mut sheet);
        }
        // No args at all
        None => {
            eprintln!("Usage:");
            eprintln!("  cargo run --release -- ext1       # launch the GUI");
            eprintln!("  cargo run --release -- <rows> <cols>");
            std::process::exit(1);
        }
    }
    Ok(())
}

#[allow(dead_code)]
use embedded::common::cell_value::{CellError, CellValue};
use embedded::common::structs::AbsCell;
use embedded::embedded_backend::single_threaded::{EmbeddedBackend, ExpressionError};
use leptos::ev::keydown;
use leptos::prelude::*;
use leptos::*;
use std::collections::VecDeque;
use std::sync::Arc;
use std::sync::Mutex;
use web_sys::KeyboardEvent;

const MAX_ROWS: usize = 999;
const MAX_COLS: usize = 999;
const DIM: usize = 10;
const DIMB: usize = 10;

// Global backend for the spreadsheet application
lazy_static::lazy_static! {
    static ref BACKEND: Mutex<EmbeddedBackend> = Mutex::new(EmbeddedBackend::new(MAX_ROWS as u16, MAX_COLS as u16));
}


fn get_column_name(mut index: usize) -> String {
    let mut name = String::new();
    index -= 1;
    loop {
        let rem = index % 26;
        name.push((b'A' + rem as u8) as char);
        if index < 26 {
            break;
        }
        index = index / 26 - 1;
    }
    name.chars().rev().collect()
}
fn parse_cell_reference(cell: String) -> (usize, usize) {
    let (col_str, row_str): (String, String) = cell.chars().partition(|c| c.is_ascii_alphabetic());

    let col = col_str.chars().fold(0, |acc, c| {
        acc * 26 + (c.to_ascii_uppercase() as u8 - b'A' + 1) as usize
    });

    let row = row_str.parse::<usize>().unwrap();

    (row, col)
}

enum EditCommand {
    ViewPort,
    Undo,
    Redo,
    EditCell {
        formula: String,
        cell_row: usize,
        cell_col: usize,
    },
    Cut {
        cell_row: usize,
        cell_col: usize,
    },
    Paste {
        formula: String,
        src_row: usize,
        src_col: usize,
        cell_row: usize,
        cell_col: usize,
    },
    Search {
        query: String,
        from_start: bool,
        current_cell: Option<(usize, usize)>,
    },
}

#[derive(Clone)]
struct CellDataF {
    value: RwSignal<String>,
    formula: RwSignal<String>,
}
fn call_backend(
    cmd: EditCommand,
    current_row: usize,
    current_col: usize,
) -> Vec<(Arc<CellDataF>, usize, usize)> {
    match cmd {
        EditCommand::ViewPort => {
            let mut result = vec![];

if let Ok(backend) = BACKEND.lock() {
    // Get cell range for the current viewport
    let top_left = AbsCell::new(current_row as i16, current_col as i16);
    let bottom_right = AbsCell::new(
        (current_row + DIM - 1) as i16,
        (current_col + DIMB - 1) as i16,
    );

    // Iterate through cells in the range
    for (cell, cell_data) in backend.get_cell_range(top_left, bottom_right) {
        let r = cell.row as usize;
        let c = cell.col as usize;

        // Convert CellValue to string representation
        let display_value = match &cell_data.value {
            Ok(CellValue::String(s)) => s.clone(),
            Ok(CellValue::Number(n)) => n.to_string(),
            Ok(CellValue::Empty) => String::new(),
            Err(CellError::DivideByZero) => "#DIV/0!".to_string(),
            Err(CellError::DependsOnNonNumeric) => "#ERROR!".to_string(),
            Err(CellError::DependsOnErr) => "#ERROR!".to_string(),
        };

        // Convert formula to string representation
        let formula = match &cell_data.formula {
            Some(expr) => format!("={:?}", expr), // Assuming Expression implements Display
            None => String::new(),
        };

        let cell_data_f = Arc::new(CellDataF {
            value: RwSignal::new(display_value),
            formula: RwSignal::new(formula),
        });

        result.push((cell_data_f, r, c));
    }
}

result
        }
        EditCommand::Undo => {
            // Use the backend's native undo functionality
            let mut result = vec![];

            // if let Ok(mut backend) = BACKEND.lock() {
            //     // Call the backend's undo method
            //     if backend.undo() {
            //         // Get the cell that was affected by the undo operation
            //         if let Some(cell) = backend.get_last_undone_cell() {
            //             let r = cell.row as usize;
            //             let c = cell.col as usize;

            //             // Get the updated value for the specific cell
            //             let value = backend.get_cell_value(cell);

            //             // Convert CellValue to string representation
            //             let display_value = match value {
            //                 Ok(CellValue::String(s)) => s.clone(),
            //                 Ok(CellValue::Number(n)) => n.to_string(),
            //                 Ok(CellValue::Empty) => String::new(),
            //                 Err(CellError::DivideByZero) => "#DIV/0!".to_string(),
            //                 Err(CellError::DependsOnNonNumeric) => "#ERROR!".to_string(),
            //                 Err(CellError::DependsOnErr) => "#ERROR!".to_string(),
            //                 _ => "#ERROR!".to_string(),
            //             };

            //             // Create the cell data
            //             let cell_data = Arc::new(CellDataF {
            //                 value: RwSignal::new(display_value.clone()),
            //                 formula: RwSignal::new(display_value),
            //             });

            //             result.push((cell_data, r, c));
            //         }
            //     }
            // }

            result
        }
        EditCommand::Redo => {
        //     // Use the backend's native redo functionality
            let mut result = vec![];

        //     if let Ok(mut backend) = BACKEND.lock() {
        //         // Call the backend's redo method
        //         if backend.redo() {
        //             // Get the cell that was affected by the redo operation
        //             if let Some(cell) = backend.get_last_redone_cell() {
        //                 let r = cell.row as usize;
        //                 let c = cell.col as usize;

        //                 // Get the updated value for the specific cell
        //                 let value = backend.get_cell_value(cell);

        //                 // Convert CellValue to string representation
        //                 let display_value = match value {
        //                     Ok(CellValue::String(s)) => s.clone(),
        //                     Ok(CellValue::Number(n)) => n.to_string(),
        //                     Ok(CellValue::Empty) => String::new(),
        //                     Err(CellError::DivideByZero) => "#DIV/0!".to_string(),
        //                     Err(CellError::DependsOnNonNumeric) => "#ERROR!".to_string(),
        //                     Err(CellError::DependsOnErr) => "#ERROR!".to_string(),
        //                     _ => "#ERROR!".to_string(),
        //                 };

        //                 // Create the cell data for the UI update
        //                 let cell_data = Arc::new(CellDataF {
        //                     value: RwSignal::new(display_value.clone()),
        //                     formula: RwSignal::new(display_value),
        //                 });

        //                 result.push((cell_data, r, c));
        //             }
        //         }
        //     }

            result
        
        }
        EditCommand::EditCell {
            formula,
            cell_row,
            cell_col,
        } => {
            let mut result = vec![];

            if let Ok(mut backend) = BACKEND.lock() {
                // Build the selected cell correctly
                let sel_cell = AbsCell::new(cell_row as i16, cell_col as i16);

                if formula.starts_with('=') {
                    let _ =backend.set_cell_formula(sel_cell, &formula[1..].to_string());

                } else {
                    match formula.parse::<f64>() {
                        Ok(num) => {
                            let _ = backend.set_cell_value(sel_cell, CellValue::Number(num));
                        }
                        Err(_) => {
                            let _ = backend.set_cell_value(sel_cell, CellValue::String(formula.clone()));
                        }
                    }
                }

                // Define the viewport
                let top_left = AbsCell::new(current_row as i16, current_col as i16);
                let bottom_right = AbsCell::new(
                    (current_row + DIM - 1) as i16,
                    (current_col + DIMB - 1) as i16,
                );

                // Iterate through the updated range
                for (cell, cell_data) in backend.get_cell_range(top_left, bottom_right) {
                    let r = cell.row as usize;
                    let c = cell.col as usize;

                    // Convert CellValue → display string
                    let display_value = match &cell_data.value {
                        Ok(CellValue::String(s)) => s.clone(),
                        Ok(CellValue::Number(n)) => n.to_string(),
                        Ok(CellValue::Empty)    => String::new(),
                        Err(CellError::DivideByZero)        => "#DIV/0!".to_string(),
                        Err(CellError::DependsOnNonNumeric) => "#ERROR!".to_string(),
                        Err(CellError::DependsOnErr)        => "#ERROR!".to_string(),
                    };

                    // Convert optional Expression → formula string
                    let formula_str = match &cell_data.formula {
                        Some(expr) => format!("={:?}", expr),  // assuming Expression: Display
                        None       => String::new(),
                    };

                    let cell_data_f = Arc::new(CellDataF {
                        value:   RwSignal::new(display_value),
                        formula: RwSignal::new(formula_str),
                    });

                    result.push((cell_data_f, r, c));
                }
            }

            result
        }

        EditCommand::Cut { cell_row, cell_col } => {
            let mut old_value = String::new();
            let mut old_formula = String::new();

            // Save current state before cutting
            if let Ok(backend) = BACKEND.lock() {
                let cell = AbsCell::new(cell_row as i16, cell_col as i16);
                let current_value = backend.get_cell_value(cell);

                // Convert current value to strings for history
                match current_value {
                    Ok(CellValue::String(s)) => {
                        old_value = s.clone();
                        old_formula = s.clone();
                    }
                    Ok(CellValue::Number(n)) => {
                        old_value = n.to_string();
                        old_formula = n.to_string();
                    }
                    Ok(CellValue::Empty) => {
                        old_value = String::new();
                        old_formula = String::new();
                    }
                    Err(_) => {
                        old_value = "#ERROR!".to_string();
                        old_formula = "#ERROR!".to_string();
                    }
                }
            }

            // Clear the cell in the backend
            if let Ok(mut backend) = BACKEND.lock() {
                let cell = AbsCell::new(cell_row as i16, cell_col as i16);
                backend.set_cell_empty(cell);
            }

            let cell_data = Arc::new(CellDataF {
                value: RwSignal::new(String::new()),
                formula: RwSignal::new(String::new()),
            });
            vec![(cell_data, cell_row, cell_col)]
        }

        EditCommand::Paste {
            formula,
            src_row: _,
            src_col: _,
            cell_row,
            cell_col,
        } => {
            let mut old_value = String::new();
            let mut old_formula = String::new();

            // Save current state before pasting
            if let Ok(backend) = BACKEND.lock() {
                let cell = AbsCell::new(cell_row as i16, cell_col as i16);
                let current_value = backend.get_cell_value(cell);

                // Convert current value to strings for history
                match current_value {
                    Ok(CellValue::String(s)) => {
                        old_value = s.clone();
                        old_formula = s.clone();
                    }
                    Ok(CellValue::Number(n)) => {
                        old_value = n.to_string();
                        old_formula = n.to_string();
                    }
                    Ok(CellValue::Empty) => {
                        old_value = String::new();
                        old_formula = String::new();
                    }
                    Err(_) => {
                        old_value = "#ERROR!".to_string();
                        old_formula = "#ERROR!".to_string();
                    }
                }
            }

            // Update the cell in the backend
            if let Ok(mut backend) = BACKEND.lock() {
                let cell = AbsCell::new(cell_row as i16, cell_col as i16);

                // Process the formula or value being pasted
                if formula.starts_with("=") {
                    let _ = backend.set_cell_formula(cell, &formula);
                } else if formula.is_empty() {
                    backend.set_cell_empty(cell);
                } else {
                    // Try to convert to appropriate type
                    if let Ok(num) = formula.parse::<f64>() {
                        backend.set_cell_value(cell, CellValue::Number(num));
                    } else if formula == "true" || formula == "false" {
                        // Since Boolean isn't a variant, store as String
                        backend.set_cell_value(cell, CellValue::String(formula.clone()));
                    } else {
                        backend.set_cell_value(cell, CellValue::String(formula.clone()));
                    }
                }

                // Get the updated value for the UI
                let result = backend.get_cell_value(cell);
                let display_value = match result {
                    Ok(CellValue::String(s)) => s.clone(),
                    Ok(CellValue::Number(n)) => n.to_string(),
                    Ok(CellValue::Empty) => String::new(),
                    Err(CellError::DivideByZero) => "#DIV/0!".to_string(),
                    Err(CellError::DependsOnNonNumeric) => "#ERROR!".to_string(),
                    Err(CellError::DependsOnErr) => "#ERROR!".to_string(),
                    _ => "#ERROR!".to_string(),
                };

                let cell_data = Arc::new(CellDataF {
                    value: RwSignal::new(display_value),
                    formula: RwSignal::new(formula),
                });
                return vec![(cell_data, cell_row, cell_col)];
            }

            // Fallback if backend lock fails
            let cell_data = Arc::new(CellDataF {
                value: RwSignal::new(formula.clone()),
                formula: RwSignal::new(formula),
            });
            vec![(cell_data, cell_row, cell_col)]
        }
        EditCommand::Search {
            query,
            from_start,
            current_cell,
        } => {
            // Use backend's search functionality
            let mut result = vec![];

            if let Ok(backend) = BACKEND.lock() {
                let found_cell = if from_start {
                    // Search from the beginning of the spreadsheet
                    backend.search_from_start(&query)
                } else if let Some((row, col)) = current_cell {
                    // Continue search from the current cell
                    let cell = AbsCell::new(row as i16, col as i16);
                    backend.search(cell, &query)
                } else {
                    // If no current cell is provided, start from the beginning
                    backend.search_from_start(&query)
                };

                // Process the search result if a cell is found
                if let Some(cell) = found_cell {
                    let r = cell.row as usize;
                    let c = cell.col as usize;

                    // Return the found cell for viewport adjustments and highlighting
                    return vec![(
                        Arc::new(CellDataF {
                            value: RwSignal::new("SEARCH_RESULT".to_string()),
                            formula: RwSignal::new("SEARCH_RESULT".to_string()),
                        }),
                        r,
                        c,
                    )];
                }
            }

            result
        }
    }
}

fn handle_edit_commands(
    cmd: EditCommand,
    table_data: &Arc<Vec<Vec<Arc<CellDataF>>>>,
    current_row: usize,
    current_col: usize,
) -> Vec<(usize, usize)> {
    let updated_cells = call_backend(cmd, current_row, current_col);
    let mut search_results = Vec::new();

    for (cell_data, target_row, target_col) in updated_cells {
        // Check if this is a search result marker
        if cell_data.value.get() == "SEARCH_RESULT" {
            // Found a search result - add to the list for navigation
            search_results.push((target_row, target_col));
            
            // Don't modify the table with the SEARCH_RESULT marker
            continue;
        }
        
        // Normal cell update - update if within current viewport
        if (target_row < current_row + DIM && target_row >= current_row)
            && (target_col < current_col + DIMB && target_col >= current_col)
        {
            let local_row = target_row - current_row;
            let local_col = target_col - current_col;

            table_data[local_row][local_col]
                .value
                .set(cell_data.value.get());
            table_data[local_row][local_col]
                .formula
                .set(cell_data.formula.get());
        }
    }
    
    search_results
}

#[component]
pub fn Spreadsheet() -> impl IntoView {
    let (current_row, set_current_row) = signal(1);
    let (current_col, set_current_col) = signal(1);
    let (source_cell, set_source_cell) = signal(String::from("A1"));
    let (check, set_check) = signal(String::new());
    let (formula, set_formula) = signal(String::new());
    let (clipboard, set_clipboard) = signal((String::new(), 1, 1));
    let (search_query, set_search_query) = signal(String::new());
    let (last_found_cell, set_last_found_cell) = signal::<Option<(usize, usize)>>(None);

    let table_data: Arc<Vec<Vec<Arc<CellDataF>>>> = Arc::new(
        (0..DIM)
            .map(|_| {
                (0..DIMB)
                    .map(|_| {
                        Arc::new(CellDataF {
                            value: RwSignal::new(String::new()),
                            formula: RwSignal::new(String::new()),
                        })
                    })
                    .collect()
            })
            .collect(),
    );
    let table_data2 = Arc::clone(&table_data);
    let table_data3 = Arc::clone(&table_data);
    let table_data4 = Arc::clone(&table_data);
    let table_data5 = Arc::clone(&table_data);
    let table_data6 = Arc::clone(&table_data);
    let table_data7 = Arc::clone(&table_data); // Additional clone for search_bar
    let table_data8 = Arc::clone(&table_data); // Additional clone for search_bar button callback

    let input_refs: Arc<Vec<Vec<NodeRef<html::Input>>>> = Arc::new(
        (0..DIM)
            .map(|_| (0..DIMB).map(|_| NodeRef::new()).collect())
            .collect(),
    );
    let input_refs2 = Arc::clone(&input_refs);
    let input_refs3 = Arc::clone(&input_refs);

    let formula_bar = move || {
        let table_datai = Arc::clone(&table_data6);
        let (r, c) = parse_cell_reference(source_cell.get());
        let cell = Arc::clone(&table_data[r - current_row.get()][c - current_col.get()]);

        view! {
            <label>"Formula: "</label>
                <input
                    type="text"
                    prop:value=formula
                    on:input=move |e| {
                        set_formula.set(event_target_value(&e));
                        cell.value.set(event_target_value(&e));
                    }
                    on:change=move |e| {
                        let input = event_target_value(&e);
                        handle_edit_commands(
                            EditCommand::EditCell {
                                formula: String::from(input),
                                cell_row: r,
                                cell_col: c,
                            },
                            &table_datai,
                            current_row.get(),
                            current_col.get(),
                        );
                    }
                />

        }
    };

    let undo = move || {
        let table_datai = Arc::clone(&table_data4);
        view! {
            <button class="undo-redo-button" on:click=move |_| {
                handle_edit_commands(
                    EditCommand::Undo,
                    &table_datai,
                    current_row.get() as usize,
                    current_col.get() as usize,
                );
            }>
                <i class="fa fa-undo"></i>
                "Undo"
            </button>
        }
    };

    let redo = move || {
        let table_datai = Arc::clone(&table_data5);

        view! {
            <button class="undo-redo-button" on:click=move |_| {

                handle_edit_commands(
                    EditCommand::Redo,
                    &table_datai,
                    current_row.get() as usize,
                    current_col.get() as usize,
                );
            }>
                <i class="fa fa-redo"></i>
                "redo"
            </button>
        }
    };

    let search_bar = move || {
        let table_datai = Arc::clone(&table_data7);
        let table_data_button = Arc::clone(&table_data8);
        view! {
            <div class="search-container">
                <label>"Search: "</label>
                <input
                    type="text"
                    placeholder="Enter search text..."
                    prop:value=search_query
                    on:input=move |e| {
                        set_search_query.set(event_target_value(&e));
                    }
                    on:keydown=move |ev: KeyboardEvent| {
                        if ev.key() == "Enter" {
                            let query = search_query.get();
                            if !query.is_empty() {
                                // When pressing Enter, start a new search
                                set_last_found_cell.set(None);
                                let search_results = handle_edit_commands(
                                    EditCommand::Search {
                                        query,
                                        from_start: true,
                                        current_cell: None,
                                    },
                                    &table_datai,
                                    current_row.get(),
                                    current_col.get(),
                                );
                                if let Some((row, col)) = search_results.first() {
                                    set_last_found_cell.set(Some((*row, *col)));
                                }
                            }
                        }
                    }
                />
                <button on:click=move |_| {
                    let query = search_query.get();
                    if !query.is_empty() {
                        let last_cell = last_found_cell.get();
                        let search_results = handle_edit_commands(
                            EditCommand::Search {
                                query: query.clone(),
                                from_start: last_cell.is_none(),
                                current_cell: last_cell,
                            },
                            &table_data_button,
                            current_row.get(),
                            current_col.get(),
                        );
                        
                        if let Some((row, col)) = search_results.first() {
                            // Found a match - navigate to it
                            set_last_found_cell.set(Some((*row, *col)));
                            
                            // Move the viewport if the cell is outside current view
                            let curr_row = current_row.get();
                            let curr_col = current_col.get();
                            
                            if *row < curr_row || *row >= curr_row + DIM || 
                               *col < curr_col || *col >= curr_col + DIMB {
                                
                                // Calculate new viewport position to center the found cell
                                let new_row = (row.saturating_sub(DIM / 2)).max(1);
                                let new_col = (col.saturating_sub(DIMB / 2)).max(1);
                                
                                set_current_row.set(new_row);
                                set_current_col.set(new_col);
                                
                                // Refresh the viewport with the new position
                                handle_edit_commands(
                                    EditCommand::ViewPort,
                                    &table_data_button,
                                    new_row,
                                    new_col,
                                );
                            }
                            
                            // Set the found cell as the selected cell
                            let cell_id = format!("{}{}", get_column_name(*col), *row);
                            set_source_cell.set(cell_id);
                            
                            // Also get the cell formula/value for display
                            if let Ok(backend) = BACKEND.lock() {
                                let cell = AbsCell::new(*row as i16, *col as i16);
                                let value = backend.get_cell_value(cell);
                                let display_value = match value {
                                    Ok(CellValue::String(s)) => s.clone(),
                                    Ok(CellValue::Number(n)) => n.to_string(),
                                    Ok(CellValue::Empty) => String::new(),
                                    Err(_) => "#ERROR!".to_string(),
                                };
                                set_formula.set(display_value);
                            }
                        } else {
                            // No match found - show an alert using web_sys
                            use web_sys::window;
                            if let Some(window) = window() {
                                let _ = window.alert_with_message("No matching results found");
                            }
                            set_last_found_cell.set(None);
                        }
                    }
                }>"Search"</button>
            </div>
        }
    };

    // Style for buttons side by side
    let buttons = {
        view! {
            <div style="display: flex; gap: 10px;">
                {undo}
                {redo}
            </div>
        }
    };

    let head = move || {
        (current_col.get()..current_col.get() + DIMB)
            .map(|col| view! { <th class="column-number">{get_column_name(col)}</th> })
            .collect::<Vec<_>>()
    };

    let body = {
        move || {
            (0..DIM)
                .map(|row| {
                    view! {
                        <tr>
                            <th class="row-number">{current_row.get() + row}</th>
                            {
                                (0..DIMB)
                                    .map(|col| {
                                        let table_data = Arc::clone(&table_data2);
                                        let irfs=Arc::clone(&input_refs2);
                                        let input_ref = irfs[row][col].clone();
                                        let cell = Arc::clone(&table_data[row][col]);
                                        let cell_id = format!(
                                            "{}{}",
                                            get_column_name(col + current_col.get()),
                                            row + current_row.get()
                                        );
                                        let is_active = cell_id == source_cell.get();
                                        let is_search_result = last_found_cell.get().map(|(r, c)| 
                                            r == row + current_row.get() && c == col + current_col.get()
                                        ).unwrap_or(false);
                                        
                                        let cell_class = move || {
                                            if is_active {
                                                if is_search_result {
                                                    "highlighted search-result"
                                                } else {
                                                    "highlighted"
                                                }
                                            } else if is_search_result {
                                                "search-result"
                                            } else {
                                                ""
                                            }
                                        };

                                        view! {
                                            <td>
                                            <input
                                                type="text"
                                                prop:value=cell.value
                                                node_ref=input_ref.clone()
                                                class={cell_class}
                                                on:click=move |_| {
                                                    set_source_cell.set(cell_id.clone());
                                                    if cell.formula.get().is_empty() {
                                                        set_formula.set(cell.value.get());
                                                    } else {
                                                        set_formula.set(cell.formula.get());
                                                    }
                                                    // mode.set(Mode::Edit);

                                                }
                                                on:input=move |e| {
                                                    set_formula.set(event_target_value(&e));
                                                }
                                                on:change=move |e| {
                                                    let input = event_target_value(&e);
                                                    handle_edit_commands(
                                                        EditCommand::EditCell {
                                                            formula: String::from(input),
                                                            cell_row: row+current_row.get(),
                                                            cell_col: col+current_col.get(),
                                                        },
                                                        &table_data,
                                                        current_row.get(),
                                                        current_col.get(),
                                                    );
                                                }
                                                // on:focus=move |_| {
                                                //     is_editing.set(true);
                                                //     // if cell2.formula.get().is_empty() {
                                                //     //     set_formula.set(cell2.value.get());
                                                //     // } else {
                                                //     //     set_formula.set(cell2.formula.get());
                                                //     // }
                                                // }
                                                // on:blur=move |_| is_editing.set(false)
                                            />
                                            </td>
                                        }
                                    })
                                    .collect_view()
                            }
                        </tr>
                    }
                })
                .collect_view()
        }
    };

    let handle_keydown = move |event: KeyboardEvent| {
        let key = event.key();
        match key.as_str() {
            "ArrowUp" | "ArrowDown" | "ArrowLeft" | "ArrowRight" => {
                event.prevent_default(); // 
                event.stop_propagation(); // 
            }
            _ => return,
        }
        let row = current_row.get();
        let col = current_col.get();
        let motion = key.as_str();

        let (old_r, old_c) = parse_cell_reference(source_cell.get());

        let (mut sel_r, mut sel_c) = (old_r, old_c);
        if event.ctrl_key() {
            let cell = &table_data3[old_r - current_row.get()][old_c - current_col.get()];

            match motion {
                "c" | "C" => {
                    // Ctrl+C
                    event.prevent_default();
                    set_clipboard.set((cell.formula.get(), old_r, old_c));
                    return;
                }
                "x" | "X" => {
                    // Ctrl+X
                    event.prevent_default();
                    set_clipboard.set((cell.formula.get(), old_r, old_c));
                    handle_edit_commands(
                        EditCommand::Cut {
                            cell_row: old_r,
                            cell_col: old_c,
                        },
                        &table_data3,
                        current_row.get(),
                        current_col.get(),
                    );
                    return;
                }
                "v" | "V" => {
                    // Ctrl+V
                    event.prevent_default();
                    let copied = clipboard.get();
                    handle_edit_commands(
                        EditCommand::Paste {
                            formula: copied.0,
                            src_row: copied.1,
                            src_col: copied.2,
                            cell_row: old_r,
                            cell_col: old_c,
                        },
                        &table_data3,
                        current_row.get(),
                        current_col.get(),
                    );
                    return;
                }
                _ => {} // do nothing for other Ctrl keys
            }
        }

        match motion {
            "ArrowUp" => {
                // key_buffer.set(String::new());
                if sel_r > 1 {
                    sel_r -= 1;
                } else {
                    return;
                }
            }
            "ArrowDown" => {
                // key_buffer.set(String::new());
                if sel_r < MAX_ROWS {
                    sel_r += 1;
                } else {
                    return;
                }
            }
            "ArrowLeft" => {
                // key_buffer.set(String::new());
                if sel_c > 1 {
                    sel_c -= 1;
                } else {
                    return;
                }
            }
            "ArrowRight" => {
                // key_buffer.set(String::new());
                if sel_c < MAX_COLS {
                    sel_c += 1;
                } else {
                    return;
                }
            }

            _ => {
                event.stop_propagation();
                event.prevent_default();
                return;
            }
        }
        //comes here

        if sel_r < DIM + row && sel_r >= row && sel_c >= col && sel_c < DIMB + col {
            set_check.update(|s| {
                let num = s.parse::<usize>().unwrap_or(0);
                *s = (num + 1).to_string();
            });
            //doesnt come here
            set_source_cell.set(format!("{}{}", get_column_name(sel_c), sel_r));
            let cell_inp = input_refs3[sel_r - row][sel_c - col];
            let cell = Arc::clone(&table_data3[sel_r - row][sel_c - col]);
            if let Some(cell_input) = cell_inp.get() {
                let _ = cell_input.focus();
            }
            if cell.formula.get().is_empty() {
                set_formula.set(cell.value.get());
            } else {
                set_formula.set(cell.formula.get());
            }
        } else {
            // doesnt come here
            let (a, b): (isize, isize) = (
                row as isize + sel_r as isize - old_r as isize,
                col as isize + sel_c as isize - old_c as isize,
            );
            set_current_row.set(a as usize);

            set_current_col.set(b as usize);

            set_source_cell.set(format!("{}{}", get_column_name(sel_c), sel_r));
            handle_edit_commands(EditCommand::ViewPort, &table_data3, a as usize, b as usize);
        }
    };
    //doesmnt come here htf is this possible

    window_event_listener(keydown, handle_keydown);

    view! {
        <div>
            <h1>"💮 Spreadsheet (A1 - AAA999)"</h1>
            <p>{move || format!("Current: row={} col={}", current_row.get(), current_col.get())}</p>
            <div style="margin-bottom: 1rem;">
                <div>
                    <label>"Selected Cell: "</label>
                    // <input type="text" bind:value=(source_cell, set_source_cell) />
                    {source_cell}
                </div>
                <div>
                    <label>"Formula: "</label>
                    {formula_bar}
                </div>
                <div>
                    <label>"Flag: "</label>
                    <input
                        type="text"
                        bind:value=(check,set_check)
                        // prop:value=formula
                        // on:input=move |e| set_formula.set(event_target_value(&e))
                    />
                </div>
                {search_bar}
                {buttons}
            </div>
            <div>
                <table>
                    <thead>
                        <tr>
                            <th>" "</th>
                            {head}
                        </tr>
                    </thead>
                    <tbody>
                        {body}
                    </tbody>
                </table>
            </div>
        </div>
    }
}

pub fn main() {
    mount_to_body(Spreadsheet);
}

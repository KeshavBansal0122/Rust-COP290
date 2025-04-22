use leptos::*;
use leptos::ev::keydown;
use leptos::prelude::*;
use std::sync::Arc;
use web_sys::KeyboardEvent;

const MAX_ROWS: usize = 999;
const MAX_COLS: usize = 999;
const DIM: usize = 10;

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
    let (col_str, row_str): (String, String) = cell
        .chars()
        .partition(|c| c.is_ascii_alphabetic());

    let col = col_str
        .chars()
        .fold(0, |acc, c| acc * 26 + (c.to_ascii_uppercase() as u8 - b'A' + 1) as usize);

    let row = row_str.parse::<usize>().unwrap();

    (row, col)
}

#[derive(Clone)]
struct CellData {
    value: RwSignal<String>,
    formula: RwSignal<String>,
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Mode {
    Navigation,
    Edit,
}


fn call_backend(value: String, abs_row: usize, abs_col: usize) -> Vec<(CellData, usize, usize)> {
    if value =="SCROLL" {
        vec![
            (
                CellData {
                    value: RwSignal::new("hello new page ".to_string()),
                    formula: RwSignal::new(String::new()),
                },
                14,
                4,
            ),
            (
                CellData {
                    value: RwSignal::new("new ppage".to_string()),
                    formula: RwSignal::new(String::new()),
                },
                15,
                1,
            ),
            (
                CellData {
                    value: RwSignal::new("page".to_string()),
                    formula: RwSignal::new("=A1+B1".to_string()),
                },
                10+abs_row,
                abs_col,
            ),
        ]
    }
    else if value == "=A1+B1" {
        vec![
            (
                CellData {
                    value: RwSignal::new("5".to_string()),
                    formula: RwSignal::new(String::new()),
                },
                4,
                4,
            ),
            (
                CellData {
                    value: RwSignal::new("3".to_string()),
                    formula: RwSignal::new(String::new()),
                },
                5,
                1,
            ),
            (
                CellData {
                    value: RwSignal::new("8".to_string()),
                    formula: RwSignal::new("=A1+B1".to_string()),
                },
                abs_row,
                abs_col,
            ),
        ]
    } else {
        vec![(
            CellData {
                value: RwSignal::new(value.clone()),
                formula: RwSignal::new(if value.starts_with('=') {
                    value.clone()
                } else {
                    String::new()
                }),
            },
            abs_row,
            abs_col,
        )]
    }
}

fn handle_motion_command(motion: &str) -> (isize, isize) {
    if motion.is_empty() {
        return (0, 0);
    }

    let (count_str, dir_char) = motion.split_at(motion.len() - 1);
    let count = count_str.parse::<isize>().unwrap_or(1);
    let dir = dir_char.chars().next().unwrap();

    match dir {
        'h' => (0, -count),
        'l' => (0, count),
        'k' => (-count, 0),
        'j' => (count, 0),
        _ => (0, 0),
    }
}

fn handle_cell_edit(
    new_value: String,
    row: usize,
    col: usize,
    table_data: &Arc<Vec<Vec<Arc<CellData>>>>,
    current_row: usize,
    current_col: usize,
) {
    let abs_row = current_row + row;
    let abs_col = current_col + col;

    let updated_cells = call_backend(new_value.clone(), abs_row, abs_col);

    for (cell_data, target_row, target_col) in updated_cells {
        if ( target_row-current_row<DIM && target_row>=current_row)
            && (target_col-current_col<DIM && target_col>=current_col)
        {
            let local_row = target_row - current_row;
            let local_col = target_col - current_col;

            table_data[local_row][local_col]
                .value
                .set(cell_data.value.get_untracked());
            table_data[local_row][local_col]
                .formula
                .set(cell_data.formula.get_untracked());
        }
    }
}

#[component]
pub fn Spreadsheet() -> impl IntoView {
    let (current_row, set_current_row) = signal(1);
    let (current_col, set_current_col) = signal(1);
    let (source_cell, set_source_cell) = signal(String::new());
    let (formula, set_formula) = signal(String::new());
    let key_buffer = RwSignal::new(String::new());
    let is_editing = RwSignal::new(false);
    let mode = RwSignal::new(Mode::Navigation);
    
    let table_data: Arc<Vec<Vec<Arc<CellData>>>> = Arc::new(
        (0..DIM)
        .map(|_| {
            (0..DIM)
            .map(|_| {
                Arc::new(CellData {
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
    let input_refs: Arc<Vec<Vec<NodeRef<html::Input>>>> = Arc::new(
        (0..DIM)
            .map(|_| (0..DIM).map(|_| NodeRef::new()).collect())
            .collect(),
    );
    let input_refs2=Arc::clone(&input_refs);
    let input_refs3=Arc::clone(&input_refs);
    
    let head = move || {
        (current_col.get()..current_col.get() + DIM)
            .map(|col| view! { <th>{get_column_name(col)}</th> })
            .collect::<Vec<_>>()
    };

    let  body = {
        move || {
            (0..DIM)
                .map(|row| {
                    view! {
                        <tr>
                            <th>{current_row.get() + row}</th>
                            {
                                (0..DIM)
                                    .map(|col| {
                                        let table_data = Arc::clone(&table_data2);
                                        let irfs=Arc::clone(&input_refs2);
                                        let input_ref = irfs[row][col].clone();
                                        let cell = Arc::clone(&table_data[row][col]);
                                        let cell2 = Arc::clone(&cell);
                                        let cell_id = format!(
                                            "{}{}",
                                            get_column_name(col + current_col.get()),
                                            row + current_row.get()
                                        );
                                        let is_active = cell_id == source_cell.get();

                                        view! {
                                            <td>
                                            <input
                                            type="text"
                                            prop:value=cell.value
                                            autofocus={is_active}
                                            node_ref=input_ref.clone()
                                            class={move || if is_active {
                                                "highlighted"
                                                    } else {
                                                        ""
                                                    }}
                                                    // if (mode.get()== Mode::Navigation);
                                                    on:click=move |_| {
                                                        set_source_cell.set(cell_id.clone());
                                                        if cell.formula.get().is_empty() {
                                                            set_formula.set(cell2.value.get());
                                                        } else {
                                                            set_formula.set(cell2.formula.get());
                                                        }
                                                        mode.set(Mode::Edit);

                                                    }
                                                    on:input=move |e| {
                                                        set_formula.set(event_target_value(&e));
                                                    }
                                                    on:change=move |e| {
                                                        let input = event_target_value(&e);
                                                        handle_cell_edit(
                                                            input.clone(),
                                                            row ,
                                                            col ,
                                                            &table_data,
                                                            current_row.get(),
                                                            current_col.get(),
                                                        );
                                                        set_formula.set(String::new());
                                                        if let Some(cell_input) = input_ref.get() {
                                                            let _ = cell_input.blur();
                                                        }
                                                        mode.set(Mode::Navigation);
                                                    }
                                                    on:focus=move |_| is_editing.set(true)
                                                    on:blur=move |_| is_editing.set(false)
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
        let mut new_row = current_row.get();
        let mut new_col = current_col.get();
        let mut scrolled = false;
        let key = event.key();
        let motion = key.as_str();
        key_buffer.set(key_buffer.get() + motion);
        // if  mode.get()==Mode::Navigation {
            // }
            
        if key_buffer.get().len() > 1000 {
            let updated_buffer = key_buffer.get()[10..].to_string();
            key_buffer.set(updated_buffer);
        }
        if mode.get()==Mode::Edit {
            if motion =="Escape" {
                event.prevent_default();
                mode.set(Mode::Navigation);
                let (row,col)=parse_cell_reference(source_cell.get());
                if let Some(cell_input) = input_refs3[row-new_row][col-new_col].get() {
                    let _ = cell_input.blur();
                }
                
            }
            
        }
        else if mode.get()==Mode::Navigation {
            let (row,col)=parse_cell_reference(source_cell.get());
            let cell = Arc::clone(&table_data3[row-new_row][col-new_col]);
            let cell_inp=input_refs3[row-new_row][col-new_col];
            if motion =="i" {
                mode.set(Mode::Edit);
                event.prevent_default();
                if let Some(cell_input) = cell_inp.get() {
                    let _ = cell_input.focus();
                }
                if cell.formula.get().is_empty() {
                    set_formula.set(cell.value.get());
                } else {
                    set_formula.set(cell.formula.get());
                }
            

            }
            if motion=="d" {
                
            }
        }
        let cell = Arc::clone(&table_data3[new_row][new_col]);
        match motion {
            "ArrowUp" => {
                key_buffer.set(String::new());
                if new_row > DIM {
                    new_row -= DIM;
                    scrolled = true;
                }
            }
            "ArrowDown" => {
                key_buffer.set(String::new());
                if new_row + 2 * DIM - 1 <= MAX_ROWS {
                    new_row += DIM;
                    scrolled = true;
                }
            }
            "ArrowLeft" => {
                key_buffer.set(String::new());
                if new_col > DIM {
                    new_col -= DIM;
                    scrolled = true;
                }
            }
            "ArrowRight" => {
                key_buffer.set(String::new());
                if new_col + 2 * DIM - 1 <= MAX_COLS {
                    new_col += DIM;
                    scrolled = true;
                }
            }
            _ => {
                scrolled=false;
                let motion = key_buffer.get();

                if !is_editing.get() && motion.chars().last().map_or(false, |c| "hjkl".contains(c)) {
                    let (offset_row, offset_col) = handle_motion_command(&motion);
                    let (r, c) = parse_cell_reference(source_cell.get());
                    let (col, row) = (
                        (c as isize + offset_col) as usize,
                        (r as isize + offset_row) as usize,
                    );

                    if 1 <= row && row <= MAX_ROWS && 1 <= col && col <= MAX_COLS {
                        set_source_cell.set(format!("{}{}", get_column_name(col), row));
                    }

                    key_buffer.set(String::new());
                }
            }
        }

        if scrolled {
            set_current_row.set(new_row);
            set_current_col.set(new_col);
            
            set_source_cell.set(format!("{}{}", get_column_name(new_col), new_row));
            handle_cell_edit(
                "SCROLL".to_string(),
                0,
                0,
                &table_data,
                current_row.get(),
                current_col.get(),
            );
        }
    };

    window_event_listener(keydown, handle_keydown);

    view! {
        <div>
            <h1>"💮 Spreadsheet (A1 - AAA999)"</h1>
            <p>{move || format!("Current: row={} col={}", current_row.get(), current_col.get())}</p>
            <div style="margin-bottom: 1rem;">
                <div>
                    <label>"Selected Cell: "</label>
                    <input type="text" bind:value=(source_cell, set_source_cell) />
                </div>
                <div>
                    <label>"Formula: "</label>
                    <input
                        type="text"
                        prop:value=formula
                        on:input=move |e| set_formula.set(event_target_value(&e))
                    />
                </div>
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
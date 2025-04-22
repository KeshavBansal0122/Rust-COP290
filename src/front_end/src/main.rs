// #[allow(dead_code)]
use leptos::*;
use leptos::ev::keydown;
use leptos::prelude::*;
use std::sync::Arc;
use web_sys::KeyboardEvent;

const MAX_ROWS: usize = 999;
const MAX_COLS: usize = 999;
const DIM: usize = 20;
const DIMB: usize = 10;

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

enum EditCommand {
    ViewPort,
    Undo,
    Redo,
    EditCell{
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
        src_row : usize,
        src_col : usize,
        cell_row: usize,
        cell_col: usize,
    },
}

#[derive(Clone)]
struct CellData {
    value: RwSignal<String>,
    formula: RwSignal<String>,
}
fn call_backend(
    cmd: EditCommand,
    current_row: usize,
    current_col: usize,
) -> Vec<(Arc<CellData>, usize, usize)> {
    match cmd {
        EditCommand::ViewPort=> {
            // No edits; return empty
            vec![]
        }
        EditCommand::Undo=> {
            vec![]
        }
        EditCommand::Redo=> {
            vec![]
        }
        EditCommand::EditCell {
            formula,
            cell_row,
            cell_col,
        } => {
            let cell_data = Arc::new(CellData {
                value: RwSignal::new(formula.clone()+"hi"),
                formula: RwSignal::new(formula),
            });
            vec![(cell_data, cell_row, cell_col)]
        }

        EditCommand::Cut {
            cell_row,
            cell_col,
        } => {
            let cell_data = Arc::new(CellData {
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
            let cell_data = Arc::new(CellData {
                value: RwSignal::new(formula.clone()),
                formula: RwSignal::new(formula),
            });
            vec![(cell_data, cell_row, cell_col)]
        }
    }
}


fn handle_edit_commands(
    cmd: EditCommand,
    table_data: &Arc<Vec<Vec<Arc<CellData>>>>,
    current_row: usize,
    current_col: usize,
) {
    let updated_cells = call_backend(cmd, current_row,current_col);

    for (cell_data, target_row, target_col) in updated_cells {
        if ( target_row<current_row+DIM && target_row>=current_row)
            && (target_col<current_col+DIMB && target_col>=current_col)
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
}


#[component]
pub fn Spreadsheet() -> impl IntoView {
    let (current_row, set_current_row) = signal(1);
    let (current_col, set_current_col) = signal(1);
    let (source_cell, set_source_cell) = signal(String::from("A1"));
    let (check, set_check) = signal(String::new());
    let (formula, set_formula) = signal(String::new());
    let (clipboard, set_clipboard) = signal((String::new(),  1, 1 ));
    // let key_buffer = RwSignal::new(String::new());
    // let is_editing = RwSignal::new(false);
    // let mode = RwSignal::new(Mode::Navigation);
    
    let table_data: Arc<Vec<Vec<Arc<CellData>>>> = Arc::new(
        (0..DIM)
        .map(|_| {
            (0..DIMB)
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
    let table_data4 = Arc::clone(&table_data);
    let table_data5 = Arc::clone(&table_data);
    let table_data6 = Arc::clone(&table_data);

    let input_refs: Arc<Vec<Vec<NodeRef<html::Input>>>> = Arc::new(
        (0..DIM)
        .map(|_| (0..DIMB).map(|_| NodeRef::new()).collect())
        .collect(),
    );
    let input_refs2=Arc::clone(&input_refs);
    let input_refs3=Arc::clone(&input_refs);


    let formula_bar = move || {
        let table_datai=Arc::clone(&table_data6);
        let (r,c)=parse_cell_reference(source_cell.get());
        let cell = Arc::clone(&table_data[r-current_row.get()][c-current_col.get()]);

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

    // Style for buttons side by side
    let buttons =  {
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

    let  body = {
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

                                        view! {
                                            <td>
                                            <input
                                                type="text"
                                                prop:value=cell.value
                                                node_ref=input_ref.clone()
                                                class={move || if is_active {
                                                    "highlighted"
                                                        } else {
                                                            ""
                                                        }
                                                    }
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
                                                    let hello=input.clone();
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
            },
            _ => {
                return
            }
        }
        let  row = current_row.get();
        let  col = current_col.get();
        let motion = key.as_str();
        
        let ( old_r,  old_c) = parse_cell_reference(source_cell.get());
        
        let (mut sel_r, mut sel_c) = (old_r,old_c);
        if event.ctrl_key() {
            let cell = &table_data3[old_r - current_row.get()][old_c - current_col.get()];
    
            match motion{
                "c" | "C" => {
                    // Ctrl+C
                    event.prevent_default();
                    set_clipboard.set((cell.formula.get(),old_r,old_c));
                    return;
                }
                "x" | "X" => {
                    // Ctrl+X
                    event.prevent_default();
                    set_clipboard.set((cell.formula.get(),old_r,old_c));
                    handle_edit_commands(
                        EditCommand::Cut { cell_row: old_r, cell_col: old_c },
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
                        EditCommand::Paste { formula: copied.0, src_row: copied.1, src_col: copied.2, cell_row: old_r, cell_col: old_c },
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
                }
                else {
                    return;
                }
            }
            "ArrowDown" => {
                

                // key_buffer.set(String::new());
                if sel_r < MAX_ROWS {
                    sel_r += 1;
                }
                else {
                    return;
                }
            }
            "ArrowLeft" => {
                // key_buffer.set(String::new());
                if sel_c > 1 {
                    sel_c -= 1;
                }
                else {
                    return;
                }
            }
            "ArrowRight" => {
                // key_buffer.set(String::new());
                if sel_c  < MAX_COLS {
                    sel_c += 1;
                }
                else {
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

        if  sel_r < DIM +row && sel_r>=row  && sel_c>= col && sel_c < DIMB +col   {
            set_check.update(|s| {
                let num = s.parse::<usize>().unwrap_or(0);
                *s = (num + 1).to_string();
            });
            //doesnt come here 
            set_source_cell.set(format!("{}{}", get_column_name(sel_c),sel_r ));
            let cell_inp=input_refs3[sel_r-row][sel_c-col];
            let cell = Arc::clone(&table_data3[sel_r-row][sel_c-col]);
            if let Some(cell_input) = cell_inp.get() {
                let _ = cell_input.focus();
            }
            if cell.formula.get().is_empty() {
                set_formula.set(cell.value.get());
            }
            else {
                set_formula.set(cell.formula.get());
            }
            

        }

        else  {
            // doesnt come here
            let (a ,b):(isize,isize)=(row as isize +sel_r as isize-old_r as isize ,col as isize +sel_c as isize -old_c as isize );
            set_current_row.set(a as usize);
            
            set_current_col.set(b as usize);
            
            set_source_cell.set(format!("{}{}", get_column_name(sel_c),sel_r ));
            handle_edit_commands(
                EditCommand::ViewPort,
                &table_data3,
                a as usize,
                b as usize,
            );
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
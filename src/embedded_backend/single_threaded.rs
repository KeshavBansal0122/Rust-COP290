use crate::common::cell_value::{CellData, CellError, CellValue};
use crate::common::structs::AbsCell;
use crate::embedded_backend::storage::Storage;
use crate::embedded_backend::structs::{Action, CellInput};
use crate::parser::formula_parser::FormulaParser;
use std::fs::File;
use std::io;
use std::path::Path;

#[derive(Debug)]
pub enum ExpressionError {
    InvalidExpression,
    CircularReference,
}
pub struct EmbeddedBackend {
    storage: Storage,
    parser: FormulaParser,
    undo_stack: Vec<Action>,
    redo_stack: Vec<Action>,
}

impl EmbeddedBackend {
    
    pub fn new(rows: u16, cols: u16) -> Self {
        EmbeddedBackend {
            storage: Storage::new(rows, cols),
            parser: FormulaParser::new(rows, cols),
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        }
    }
    
    pub fn from_file(file: &File) -> io::Result<Self> {
        let storage = Storage::from_file(file)?;
        Ok(EmbeddedBackend {
            storage,
            parser: FormulaParser::new(0, 0),
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        })
    }
    
    pub fn save_to_file(&self, file: &File) -> io::Result<()> {
        self.storage.serialize_to_file(file)
    }
    pub fn set_cell_empty(&mut self, cell: AbsCell) {
        self.set_cell_value(cell, CellValue::Empty);
    }
    
    pub fn set_cell_value(&mut self, cell: AbsCell, value: CellValue) {
        let old = self.storage.get_input(cell);
        let new = CellInput::Value(value.clone());
        let action = Action {
            cell,
            old_value: old,
            new_value: new,
        };
        self.storage.set_value(cell, value);
        self.undo_stack.push(action);
        if !self.redo_stack.is_empty() {
            self.redo_stack.clear();
        }
    }
    
    pub fn get_cell_value(&self, cell: AbsCell) -> &Result<CellValue, CellError> {
        self.storage.get_value(cell)
    }

    pub fn get_cell_formula(&self, cell: AbsCell) -> Option<String> {
        self.storage.get_cell_formula(cell)
    }
    
    pub fn get_cell_range(&self,
                          top_left: AbsCell,
                          bottom_right: AbsCell
    ) -> impl Iterator<Item = (AbsCell, &CellData)> {
        self.storage.get_value_range_full(top_left, bottom_right)
    }
    
    pub fn set_cell_formula(&mut self, cell: AbsCell, formula: &str) -> Result<(), ExpressionError> {
        let new = self.parser.parse(formula, cell).map_err(|_| ExpressionError::InvalidExpression)?;
        let old = self.storage.get_input(cell);
        
        if !self.storage.set_expression(cell, new) {
            Err(ExpressionError::CircularReference)
        } else { 
            let action = Action {
                cell,
                old_value: old,
                new_value: self.storage.get_input(cell),
            };
            self.undo_stack.push(action);
            if !self.redo_stack.is_empty() {
                self.redo_stack.clear();
            }
            Ok(())
        }
    }
    
    /// Returns true if the undo stack was not empty and undo actually happened
    pub fn undo(&mut self) -> bool {
        if let Some(action) = self.undo_stack.pop() {
            let old = &action.old_value;
            match old {
                CellInput::Value(value) => {
                    self.storage.set_value(action.cell, value.clone());
                }
                CellInput::Formula(formula) => {
                    self.set_cell_formula(action.cell, formula)
                        .expect("Panic from undo not expected");
                }
            }
            self.redo_stack.push(action);
            true
        } else {
            false
        }
    }
    
    /// Returns true if the redo stack was not empty and redo actually happened
    pub fn redo(&mut self) -> bool {
        if let Some(action) = self.redo_stack.pop() {
            let new = &action.new_value;
            match new {
                CellInput::Value(value) => {
                    self.storage.set_value(action.cell, value.clone());
                }
                CellInput::Formula(formula) => {
                    self.set_cell_formula(action.cell, formula)
                        .expect("Panic from redo not expected");
                }
            }
            self.undo_stack.push(action);
            true
        } else {
            false
        }
    }
    
    pub fn copy_cell_expression(&mut self, from: AbsCell, to: AbsCell) -> Result<(), ExpressionError> {
        if self.storage.copy_cell_expression(from, to) {
            Ok(())
        } else {
            Err(ExpressionError::CircularReference)
        }
    }
    
    
    pub fn search(&self, cell: AbsCell, to_search: &str) -> Option<AbsCell> {
        self.storage.search(cell, to_search)
    }
    
    pub fn search_from_start(&self, to_search: &str) -> Option<AbsCell> {
        self.storage.search_from_start(to_search)
    }
    
    /// Saves a rectangular range of cells to a CSV file.
    ///
    /// # Arguments
    /// * `top_left` - The top-left cell of the range.
    /// * `bottom_right` - The bottom-right cell of the range.
    /// * `file_path` - The path to the CSV file where the data will be saved.
    ///
    /// # Returns
    /// * `Result<(), std::io::Error>` - Ok if successful, Err if an error occurs.
    pub fn save_range_to_csv(
        &self,
        top_left: AbsCell,
        bottom_right: AbsCell,
        file_path: &Path,
    ) -> Result<(), io::Error> {
        let mut writer = csv::Writer::from_path(file_path)?;

        for row in top_left.row..=bottom_right.row {
            let mut csv_row = Vec::new();
            for col in top_left.col..=bottom_right.col {
                let cell = AbsCell::new(row, col);
                let value = self.get_cell_value(cell);
                let cell_content = match value {
                    Ok(CellValue::Empty) => "".to_string(),
                    Ok(CellValue::Number(num)) => num.to_string(),
                    Ok(CellValue::String(text)) => text.clone(),
                    Err(_) => "#ERROR".to_string(),
                };
                csv_row.push(cell_content);
            }
            writer.write_record(csv_row)?;
        }

        writer.flush()?;
        Ok(())
    }
}

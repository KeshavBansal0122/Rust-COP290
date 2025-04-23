use crate::common::cell_value::{CellError, CellValue};
use crate::common::structs::AbsCell;
use crate::embedded_backend::storage::Storage;
use crate::embedded_backend::structs::{Action, CellInput};
use crate::parser::formula_parser::FormulaParser;

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

    pub fn get_cell_range(
        &self,
        top_left: AbsCell,
        bottom_right: AbsCell,
    ) -> impl Iterator<Item = (AbsCell, &Result<CellValue, CellError>)> {
        self.storage.get_value_range_full(top_left, bottom_right)
    }

    pub fn set_cell_formula(
        &mut self,
        cell: AbsCell,
        formula: &str,
    ) -> Result<(), ExpressionError> {
        let new = self
            .parser
            .parse(formula, cell)
            .map_err(|_| ExpressionError::InvalidExpression)?;
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

    pub fn copy_cell_expression(
        &mut self,
        from: AbsCell,
        to: AbsCell,
    ) -> Result<(), ExpressionError> {
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
    /// Returns the cell affected by the last undo operation
    pub fn get_last_undone_cell(&self) -> Option<AbsCell> {
        if !self.redo_stack.is_empty() {
            // The last action that was undone is now at the top of the redo stack
            Some(self.redo_stack.last().unwrap().cell)
        } else {
            None
        }
    }
    /// Returns the cell affected by the last redo operation
    pub fn get_last_redone_cell(&self) -> Option<AbsCell> {
        if !self.undo_stack.is_empty() {
            // The last action that was redone is now at the top of the undo stack
            Some(self.undo_stack.last().unwrap().cell)
        } else {
            None
        }
    }
}

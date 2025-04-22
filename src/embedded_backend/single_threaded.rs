use crate::common::cell_value::{CellError, CellValue};
use crate::common::structs::AbsCell;
use crate::embedded_backend::storage::Storage;
use crate::parser::formula_parser::FormulaParser;

pub enum ExpressionError {
    InvalidExpression,
    CircularReference,
}
pub struct EmbeddedBackend {
    storage: Storage,
    parser: FormulaParser
}

impl EmbeddedBackend {
    
    pub fn new(rows: u16, cols: u16) -> Self {
        EmbeddedBackend {
            storage: Storage::new(),
            parser: FormulaParser::new(rows, cols)
        }
    }
    pub fn set_cell_empty(&mut self, cell: AbsCell) {
        self.set_cell_value(cell, CellValue::Empty);
    }
    
    pub fn set_cell_value(&mut self, cell: AbsCell, value: CellValue) {
        self.storage.set_value(cell, value);
    }
    
    pub fn get_cell_value(&self, cell: AbsCell) -> &Result<CellValue, CellError> {
        self.storage.get_value(cell)
    }
    
    pub fn get_cell_range(&self,
                          top_left: AbsCell,
                          bottom_right: AbsCell
    ) -> impl Iterator<Item = (AbsCell, &Result<CellValue, CellError>)> {
        self.storage.get_value_range_full(top_left, bottom_right)
    }
    
    pub fn set_cell_formula(&mut self, cell: AbsCell, formula: &str) -> Result<(), ExpressionError> {
        let new = self.parser.parse(formula, cell).map_err(|_| ExpressionError::InvalidExpression)?;
        
        if !self.storage.set_expression(cell, new) {
            Err(ExpressionError::CircularReference)
        } else { 
            Ok(())
        }
    }
    
    
}
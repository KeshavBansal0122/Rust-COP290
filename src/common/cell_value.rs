use serde::{Deserialize, Serialize};
use crate::common::expression::Expression;

#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum CellError {
    DivideByZero,
    DependsOnNonNumeric,
    DependsOnErr
}
#[derive(Clone, Debug, Default, PartialEq)]
pub enum CellValue {
    #[default]
    Empty,
    String(String),
    Number(f64),
}

#[derive(Clone, Debug, PartialEq)]
pub struct CellData {
    pub value: Result<CellValue, CellError>,
    pub formula: Option<Expression>,
}

impl Default for CellData {
    fn default() -> Self {
        CellData {
            value: Ok(CellValue::Empty),
            formula: None,
        }
    }
}
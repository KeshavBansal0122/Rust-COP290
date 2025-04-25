use crate::common::expression::Expression;
use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum CellError {
    DivideByZero,
    DependsOnNonNumeric,
    DependsOnErr,
}
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub enum CellValue {
    #[default]
    Empty,
    String(String),
    Number(f64),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
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

impl CellData {
    pub fn default_instance() -> &'static CellData {
        static DEFAULT_CELL: CellData = CellData {
            value: Ok(CellValue::Empty),
            formula: None,
        };
        &DEFAULT_CELL
    }
}

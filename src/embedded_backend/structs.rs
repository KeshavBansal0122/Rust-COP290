use crate::common::cell_value::CellValue;
use crate::common::structs::AbsCell;

pub enum CellInput {
    Value(CellValue),
    Formula(String),
}

pub struct Action {
    pub cell: AbsCell,
    pub old_value: CellInput,
    pub new_value: CellInput,
}
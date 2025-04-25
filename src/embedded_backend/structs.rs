//! The structs in this module are used to model the single interaction of the user with the spreadsheet.
//! This can be used to implement collaboration, where interactions are send and each client calculates the
//! effect of that interaction on its own end
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

use crate::common::structs::AbsCell;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Stores the data necessary for the cell graph -> The cells that depend on this cell.
///
/// This is stored in a hashSet. This allows to easily add and remove cells, and quick cycle detection
/// due to O(1) lookup time. Also don't have to worry about duplicates from recursive formulas
#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct CellMetadata {
    pub dependents: HashSet<AbsCell>,
}

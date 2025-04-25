use crate::common::structs::AbsCell;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Stores the data necessary for the cell graph -> the edges and
#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct CellMetadata {
    pub dependents: HashSet<AbsCell>,
}

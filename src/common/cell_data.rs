use std::collections::HashSet;
use serde::{Deserialize, Serialize};
use crate::common::structs::AbsCell;


/// Stores the data necessary for the cell graph -> the edges and 
#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct CellMetadata {
    pub dependents: HashSet<AbsCell>,
}
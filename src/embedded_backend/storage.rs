use crate::common::cell_data::CellMetadata;
use crate::common::cell_value::{CellData, CellError, CellValue};
use crate::common::expression::Expression;
use crate::common::structs::AbsCell;
use crate::embedded_backend::calc_engine::evaluate;
use std::collections::{BTreeMap, HashMap, HashSet};

#[derive(Debug, Default)]
pub struct Storage {
    values: BTreeMap<AbsCell, CellData>,
    graph: HashMap<AbsCell, CellMetadata>,
    // graph: DiGraphMap<AbsCell, (), RandomState>
}


impl Storage {
    
    pub fn new() -> Self {
        Storage {
            values: BTreeMap::new(),
            graph: HashMap::new(),
        }
    }
    pub fn get_value(&self, cell: AbsCell) -> &Result<CellValue, CellError> {
        let x = self.values.get(&cell).map(|cell_data| &cell_data.value);
        x.unwrap_or(&Ok(CellValue::Empty))
    }
    

    /// Sets the value of the cell and recomputes its dependants
    pub fn set_value(&mut self, cell: AbsCell, value: CellValue) {
        if value == CellValue::Empty {
            self.values.remove(&cell);
        } else { 
            let cell_data = self.values.entry(cell).or_default();
            cell_data.value = Ok(value);
        }
        self.graph.remove(&cell);
        self.recalculate_cell(cell);
    }
    
    
    
    /// Gives a sparse iterator over a closed rectangle of cells. Returns only the cells that 
    /// are explicitly stored instead of their default values
    /// 
    /// Iterator over a large range can have a lot of empty cells, this will return only the 
    /// non-empty ones
    /// 
    /// 
    /// 
    /// # Arguments 
    /// 
    /// * `top_left`: topleft of the rectangle
    /// * `bottom_right`: bottom right of the rectangle
    /// 
    /// returns: The iterator over the cells in range. 
    /// empty if bottom_right.row and col >= top_left.row and col not satisfied
    pub fn get_value_range_sparse(&self,
                                  top_left: AbsCell,
                                  bottom_right: AbsCell
    ) -> impl Iterator<Item=(AbsCell, &Result<CellValue, CellError>)> {
        SparseRangeIter::new(top_left, bottom_right, &self.values)
    }
    
    pub fn get_value_range_full(&self,
                                top_left: AbsCell,
                                bottom_right: AbsCell
    ) -> impl Iterator<Item=(AbsCell, &Result<CellValue, CellError>)> {
        FullRangeIter::new(top_left, bottom_right, &self.values)
    }
    
    fn get_dep(&self, cell: AbsCell) -> &HashSet<AbsCell> {
        &self.graph.get(&cell).unwrap().dependents
    }
    
    pub fn get_expression(&self, cell: AbsCell) -> Option<&Expression> {
        let exp = self.values.get(&cell)?;
        exp.formula.as_ref()
    }
    
    fn recalculate_cell(&mut self, cell: AbsCell) {
        let exp = self.values.get(&cell);
        if let Some(exp) = exp {
            if let Some(exp) = &exp.formula {
                let res = evaluate(self, cell, exp).map(CellValue::Number);
                self.values.entry(cell).or_default().value = res;
            }
        }
    }
    
    fn update_cells(&mut self, cell: AbsCell) {
        let mut stack = vec![cell];
        let mut dirty_parents = HashMap::new();
        dirty_parents.insert(cell, 0u8);
        
        //dirty marking
        while let Some(top) = stack.pop() {
            for x in self.get_dep(top) {
                let count = dirty_parents.entry(*x).or_default();
                if *count == 0 {
                    stack.push(*x);
                }
                *count += 1;
            }   
        }
        
        //now start recalculation from here
        stack.push(cell);
        
        while let Some(top) = stack.pop() {
            self.recalculate_cell(top);
            
            for x in self.get_dep(top) {
                let cnt = dirty_parents.get_mut(x).expect("complete chain already inserted");
                *cnt -= 1;
                if *cnt == 0 {
                    stack.push(*x);
                }
            }
        }
    }
    
    /// Updates the graph according to the new expression. 
    /// Does not add the expression if it causes a ciruclar dependancy
    /// 
    /// Also updates the values of the cells according to the expression if no circular dependancy was cause
    /// 
    /// # Arguments 
    /// 
    /// * `cell`: 
    /// * `expression`: 
    /// 
    /// returns: bool
    pub fn set_expression(&mut self, cell: AbsCell, expression: Expression) -> bool {
        let cell_data = self.values.get(&cell);
        
        //remove old edges
        if let Some(cell_data) = cell_data {
            if let Some(old_exp) = &cell_data.formula {
                
                //remove
                let mut referenced_cells = HashSet::new();
                Self::collect_referenced_cells(old_exp, cell, &mut referenced_cells);

                for referenced_cell in referenced_cells {
                    if let Some(metadata) = self.graph.get_mut(&referenced_cell) {
                        metadata.dependents.remove(&cell);
                    }
                }
            }
        }

        //add new
        let mut referenced_cells = HashSet::new();
        Self::collect_referenced_cells(&expression, cell, &mut referenced_cells);

        for referenced_cell in referenced_cells {
            self.graph
                .entry(referenced_cell)
                .or_default()
                .dependents
                .insert(cell);
        }
        
        if self.check_circular(cell) {
            
            //remove
            let mut referenced_cells = HashSet::new();
            Self::collect_referenced_cells(&expression, cell, &mut referenced_cells);

            for referenced_cell in referenced_cells {
                if let Some(metadata) = self.graph.get_mut(&referenced_cell) {
                    metadata.dependents.remove(&cell);
                }
            }
            if let Some(cell_data) = cell_data {
                if let Some(old_exp) = &cell_data.formula {
                    
                    //add old
                    let mut referenced_cells = HashSet::new();
                    Self::collect_referenced_cells(old_exp, cell, &mut referenced_cells);

                    for referenced_cell in referenced_cells {
                        self.graph
                            .entry(referenced_cell)
                            .or_default()
                            .dependents
                            .insert(cell);
                    }
                }
            }
            return false
        }
        
        let cell_data = self.values.entry(cell).or_default();
        cell_data.formula = Some(expression);
        self.update_cells(cell);
        true
    }
   
    pub fn add_edges(&mut self, cell: AbsCell, expression: &Expression) {
        let mut referenced_cells = HashSet::new();
        Self::collect_referenced_cells(expression, cell, &mut referenced_cells);

        for referenced_cell in referenced_cells {
            self.graph
                .entry(referenced_cell)
                .or_default()
                .dependents
                .insert(cell);
        }
    }
    
    pub fn remove_edges(&mut self, cell: AbsCell, expression: &Expression) {
        let mut referenced_cells = HashSet::new();
        Self::collect_referenced_cells(expression, cell, &mut referenced_cells);

        for referenced_cell in referenced_cells {
            if let Some(metadata) = self.graph.get_mut(&referenced_cell) {
                metadata.dependents.remove(&cell);
            }
        }
    }

    fn collect_referenced_cells(expression: &Expression, cell: AbsCell, referenced_cells: &mut HashSet<AbsCell>) {
        match expression {
            Expression::Cell(rel_cell) => {
                referenced_cells.insert(rel_cell.to_abs(cell));
            }
            Expression::BinaryOp(lhs, _, rhs) => {
                Self::collect_referenced_cells(lhs, cell, referenced_cells);
                Self::collect_referenced_cells(rhs, cell, referenced_cells);
            }
            Expression::RangeFunction(_, range) => {
                let top_left = range.top_left.to_abs(cell);
                let bottom_right = range.bottom_right.to_abs(cell);
                for row in top_left.row..=bottom_right.row {
                    for col in top_left.col..=bottom_right.col {
                        referenced_cells.insert(AbsCell::new(row, col));
                    }
                }
            }
            Expression::Sleep(inner) => {
                Self::collect_referenced_cells(inner, cell, referenced_cells);
            }
            Expression::Number(_) => {}
        }
    }

    /// # Arguments 
    /// 
    /// * `cell`: the cell to check for circular dependency
    /// 
    /// returns: if the given cell is in a loop 
    pub fn check_circular(&self, cell: AbsCell) -> bool {
        let mut stack = vec![cell];
        let mut found = HashSet::new();
        while let Some(top) = stack.pop() {
            for &x in self.get_dep(top) {
                if x == cell {
                    return true;
                }
                
                //  found for the first time
                if !found.contains(&x) {
                    stack.push(x);
                    found.insert(x);
                }
                
            }
        }
        false
    }
}


struct SparseRangeIter<'a> {
    top_left: AbsCell,
    bottom_right: AbsCell,
    values: &'a BTreeMap<AbsCell, CellData>,
    value_iter: std::collections::btree_map::Range<'a, AbsCell, CellData>,
    current_row: AbsCell
}

impl<'a> SparseRangeIter<'a> {
    fn new(top_left: AbsCell, bottom_right: AbsCell, values: &'a BTreeMap<AbsCell, CellData>) -> Self {
        let is_valid = top_left.row <= bottom_right.row && top_left.col <= bottom_right.col;

        let value_iter = if is_valid {
            values.range(top_left..=bottom_right)
        } else {
            values.range(bottom_right..bottom_right) //empty range
        };

        let current_row = if is_valid {
            top_left
        }  else {
            bottom_right
        }; //will stop on the first instance of .next

        SparseRangeIter {
            top_left,
            bottom_right,
            values,
            value_iter,
            current_row
        }
    }
}

impl<'a> Iterator for SparseRangeIter<'a> {
    type Item = (AbsCell, &'a Result<CellValue, CellError>);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let val = self.value_iter.next();
            if let Some((cell, value)) = val {
                return Some((*cell, &value.value));

            } else if self.current_row.row != self.bottom_right.row {
                self.current_row.row += 1;
                let left = AbsCell::new(self.current_row.row, self.top_left.col);
                let right = AbsCell::new(self.current_row.row, self.bottom_right.col);
                self.value_iter = self.values.range(left..=right);

            } else {
                return None;
            }
        }

    }
}

struct FullRangeIter<'a> {
    top_left: AbsCell,
    bottom_right: AbsCell,
    values: &'a BTreeMap<AbsCell, CellData>,
    value_iter: std::collections::btree_map::Range<'a, AbsCell, CellData>,
    current_cell: AbsCell,
    next_value: Option<(&'a AbsCell, &'a CellData)>
}

impl<'a> FullRangeIter<'a> {
    fn new(top_left: AbsCell, bottom_right: AbsCell, values: &'a BTreeMap<AbsCell, CellData>) -> Self {
        let is_valid = top_left.row <= bottom_right.row && top_left.col <= bottom_right.col;

        let mut value_iter = if is_valid {
            values.range(top_left..=bottom_right)
        } else {
            values.range(bottom_right..bottom_right) //empty range
        };

        let current_cell = if is_valid {
            top_left
        }  else {
            bottom_right
        }; //will stop on the first instance of .next
        let next_value = value_iter.next();
        FullRangeIter {
            top_left,
            bottom_right,
            values,
            value_iter,
            current_cell,
            next_value
        }
    }
}

impl<'a> Iterator for FullRangeIter<'a> {
    type Item = (AbsCell, &'a Result<CellValue, CellError>);

    fn next(&mut self) -> Option<Self::Item> {
        // Check if we've gone beyond the bottom-right boundary
        if self.current_cell.row > self.bottom_right.row {
            return None;
        }

        let result_cell = self.current_cell;

        // Advance the current_cell for the next iteration
        if self.current_cell.col < self.bottom_right.col {
            self.current_cell.col += 1;
        } else {
            self.current_cell.col = self.top_left.col;
            self.current_cell.row += 1;
        }

        // Check if the next value from the BTree matches our current cell
        match self.next_value {
            Some((cell, data)) if *cell == result_cell => {
                // Consume this value and fetch the next one for future iterations
                self.next_value = self.value_iter.next();
                Some((result_cell, &data.value))
            },
            _ => {
                // Either no next value or it doesn't match our current cell
                // Return an empty cell
                Some((result_cell, &Ok(CellValue::Empty)))
            }
        }
    }
}
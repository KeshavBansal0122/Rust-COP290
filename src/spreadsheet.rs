use crate::function::{eval_binary, eval_range};
use crate::myparser::MyParser;
use std::collections::{HashMap, HashSet};
use std::thread;
use std::time::Duration;

type ChildNormalType = (String, HashSet<(u16, u16)>);
type ChildRangeType = (String, (u16, u16), (u16, u16));
#[derive(Debug, Clone, PartialEq, Copy)]
pub enum Cell {
    Value(i32),
    Err,
}

impl Cell {
    pub fn new() -> Self {
        Cell::Value(0)
    }
}

/// Convert a 1-based column index into letters (1→"A", 27→"AA")
fn col_to_letter(mut n: usize) -> String {
    let mut s = String::new();
    while n > 0 {
        n -= 1;
        s.push((b'A' + (n % 26) as u8) as char);
        n /= 26;
    }
    s.chars().rev().collect()
}

pub struct Spreadsheet {
    pub rows: usize,
    pub cols: usize,
    pub parents_normal: HashMap<(u16, u16), HashSet<(u16, u16)>>,
    pub child_normal: HashMap<(u16, u16), ChildNormalType>,
    pub child_range: HashMap<(u16, u16), ChildRangeType>,
    pub cells: Vec<Vec<Cell>>,
}

impl Spreadsheet {
    pub fn new(rows: usize, cols: usize) -> Self {
        let mut cells = Vec::with_capacity(rows + 1);
        for _ in 0..=rows {
            cells.push(vec![Cell::new(); cols + 1]);
        }
        Spreadsheet {
            rows,
            cols,
            parents_normal: HashMap::new(),
            child_normal: HashMap::new(),
            child_range: HashMap::new(),
            cells,
        }
    }

    /// Return `Some(v)` if cell is a value, or `None` if it's `Err` or out of bounds.
    fn get_val(&self, (c, r): (u16, u16)) -> Option<i32> {
        if r as usize <= self.rows && c as usize <= self.cols {
            match &self.cells[r as usize][c as usize] {
                Cell::Value(v) => Some(*v),
                Cell::Err => None,
            }
        } else {
            None
        }
    }

    /// Set a cell’s formula or literal.  Abort (no change) on any parse error,
    /// except when `/0` in a binary formula, which writes `Err`.
    pub fn set_cell(&mut self, coord: (u16, u16), expr: &str) -> u8 {
        if coord.1 as usize > self.rows || coord.0 as usize > self.cols {
            return 1; // Invalid cell
        }
        // Check for out-of-bounds coordinates
        //print parents and child dependencies before setting cell
        // println!("parents: {:?}", self.parents_normal);
        // println!("child: {:?}", self.child_normal);
        // println!("child_range: {:?}", self.child_range);
        // 1) clear old dependencies but save them first
        let old_cell_value = self.cells[coord.1 as usize][coord.0 as usize];
        let mut removed_from_parents = Vec::new();
        let old_child_normal = self.child_normal.remove(&coord);
        let old_child_range = self.child_range.remove(&coord);

        // Track which entries we're removing from parents_normal
        for (parent_coord, deps) in self.parents_normal.iter_mut() {
            if deps.contains(&coord) {
                removed_from_parents.push((*parent_coord, coord));
                deps.remove(&coord);
            }
        }

        let expr = expr.trim();

        // 1a) Check for SLEEP function with a constant value: "SLEEP(5)"
        if expr.starts_with("SLEEP(") && expr.ends_with(")") {
            let arg_str = &expr[6..expr.len() - 1];
            // Check if the argument contains a colon, which indicates a range
            if arg_str.contains(':') {
                // Range is not allowed in SLEEP function
                // Restore old child dependencies
                if let Some(old_normal) = old_child_normal {
                    self.child_normal.insert(coord, old_normal);
                }
                if let Some(old_range) = old_child_range {
                    self.child_range.insert(coord, old_range);
                }
                // Restore parents
                for (parent_coord, child_coord) in removed_from_parents {
                    self.parents_normal
                        .entry(parent_coord)
                        .or_default()
                        .insert(child_coord);
                }
                return 3; // unrecognized cmd - range not allowed in SLEEP
            }
            // Try to parse as a literal integer
            if let Ok(sleep_time) = arg_str.parse::<i32>() {
                // Direct sleep with constant
                if sleep_time > 0 {
                    thread::sleep(Duration::from_secs(sleep_time as u64));
                }
                self.cells[coord.1 as usize][coord.0 as usize] = Cell::Value(sleep_time);
                self.recalc_dependents(coord);
                return 0;
            }
            // Try to parse as a cell reference
            else if let Some(ref_cell) = MyParser::cell_name_to_coord(arg_str) {
                // Add dependency tracking
                self.parents_normal
                    .entry(ref_cell)
                    .or_default()
                    .insert(coord);
                let mut refs = HashSet::new();
                refs.insert(ref_cell);
                self.child_normal.insert(coord, (expr.to_string(), refs));

                if self.has_cycle_from(coord) {
                    //reverse the parents normal and child normal changes done above
                    self.parents_normal
                        .entry(ref_cell)
                        .or_default()
                        .remove(&coord);
                    self.child_normal.remove(&coord);
                    //get the old values in these hashmaps
                    for (parent_coord, child_coord) in removed_from_parents {
                        self.parents_normal
                            .entry(parent_coord)
                            .or_default()
                            .insert(child_coord);
                    }
                    // Restore old child dependencies
                    if let Some(old_normal) = old_child_normal {
                        self.child_normal.insert(coord, old_normal);
                    }
                    if let Some(old_range) = old_child_range {
                        self.child_range.insert(coord, old_range);
                    }
                    return 4;
                }
                match self.get_val(ref_cell) {
                    Some(sleep_time) => {
                        // Sleep using the referenced cell's value
                        if sleep_time > 0 {
                            thread::sleep(Duration::from_secs(sleep_time as u64));
                        }
                        self.cells[coord.1 as usize][coord.0 as usize] = Cell::Value(sleep_time);
                    }
                    None => {
                        self.cells[coord.1 as usize][coord.0 as usize] = Cell::Err;
                    }
                }
                self.recalc_dependents(coord);
                return 0;
            }
        }

        // 2a) Binary: "A1+2", "3/0", etc.
        if let Some((op_char, lhs_s, rhs_s)) = MyParser::split_binary(expr) {
            let op_code = match op_char {
                '+' => 1,
                '-' => 2,
                '*' => 3,
                '/' => 5,
                _ => return 3, // unrecognized cmd  invalid operator
            };

            // Evaluate lhs
            let a = if let Some(c) = MyParser::cell_name_to_coord(lhs_s) {
                match self.get_val(c) {
                    Some(val) => Cell::Value(val),
                    //for None return Err
                    None => Cell::Err,
                }
            } else {
                match lhs_s.parse::<i32>() {
                    Ok(val) => Cell::Value(val),
                    Err(_) => return 3,
                }
            };

            // Evaluate rhs
            let b = if let Some(c) = MyParser::cell_name_to_coord(rhs_s) {
                match self.get_val(c) {
                    Some(val) => Cell::Value(val),
                    None => Cell::Err,
                }
            } else {
                match rhs_s.parse::<i32>() {
                    Ok(val) => Cell::Value(val),
                    Err(_) => return 3,
                }
            };

            let new_cell =
                if op_code == 5 && b == Cell::Value(0) || a == Cell::Err || b == Cell::Err {
                    Cell::Err
                }
                //else if both are values
                else if let (Cell::Value(va), Cell::Value(vb)) = (a, b) {
                    if let Some(v) = eval_binary(op_code, va, vb) {
                        Cell::Value(v)
                    } else {
                        return 5; // division by zero
                    }
                } else {
                    return 3;
                };
            let mut updated_parents = Vec::new();
            // adding new dependencies
            let mut refs = HashSet::new();
            if let Some(c) = MyParser::cell_name_to_coord(lhs_s) {
                updated_parents.push((c, coord));
                self.parents_normal.entry(c).or_default().insert(coord);
                refs.insert(c);
            }
            if let Some(c) = MyParser::cell_name_to_coord(rhs_s) {
                updated_parents.push((c, coord));
                self.parents_normal.entry(c).or_default().insert(coord);
                refs.insert(c);
            }
            self.child_normal.insert(coord, (expr.to_string(), refs));
            // Check for cycles
            if self.has_cycle_from(coord) {
                // Reverse the parents_normal and child_normal changes done above
                for (parent, child) in updated_parents {
                    self.parents_normal
                        .entry(parent)
                        .or_default()
                        .remove(&child);
                }
                self.child_normal.remove(&coord);
                // Restore old child dependencies
                if let Some(old_normal) = old_child_normal {
                    self.child_normal.insert(coord, old_normal);
                }
                if let Some(old_range) = old_child_range {
                    self.child_range.insert(coord, old_range);
                }
                // Restore parents
                for (parent_coord, child_coord) in removed_from_parents {
                    self.parents_normal
                        .entry(parent_coord)
                        .or_default()
                        .insert(child_coord);
                }
                // Keep the old cell value
                self.cells[coord.1 as usize][coord.0 as usize] = old_cell_value;
                return 4;
            }
            self.cells[coord.1 as usize][coord.0 as usize] = new_cell;
            self.recalc_dependents(coord);
            return 0;
        }

        // 2b) Range: "SUM(A1:B3)"
        if let Some((func, start, end)) = MyParser::parse_range(expr) {
            // if start > end => return 3, and check in bounds
            if start.0 > end.0
                || start.1 > end.1
                || start.0 > self.cols as u16
                || start.1 > self.rows as u16
                || end.0 > self.cols as u16
                || end.1 > self.rows as u16
            {
                return 3; // unrecognized cmd
            }
            // Add the range dependency
            self.child_range
                .insert(coord, (expr.to_string(), start, end));

            // Check for cycles that might be created by this range reference
            if self.has_cycle_from(coord) {
                // Cycle detected - remove the range dependency we just added
                self.child_range.remove(&coord);
                // Restore old child dependencies
                if let Some(old_normal) = old_child_normal {
                    self.child_normal.insert(coord, old_normal);
                }
                if let Some(old_range) = old_child_range {
                    self.child_range.insert(coord, old_range);
                }
                // Restore parents
                for (parent_coord, child_coord) in removed_from_parents {
                    self.parents_normal
                        .entry(parent_coord)
                        .or_default()
                        .insert(child_coord);
                }
                // Keep the old cell value
                self.cells[coord.1 as usize][coord.0 as usize] = old_cell_value;
                return 4;
            }
            // No cycle, proceed with evaluation
            if let Some(v) = eval_range(func, start, end, |c| self.get_val(c)) {
                self.cells[coord.1 as usize][coord.0 as usize] = Cell::Value(v);
            } else {
                self.cells[coord.1 as usize][coord.0 as usize] = Cell::Err;
            }
            self.recalc_dependents(coord);
            return 0;
        }

        // 2c) Single reference: "C5"
        if let Some(c) = MyParser::cell_name_to_coord(expr) {
            // Add dependency tracking
            self.parents_normal.entry(c).or_default().insert(coord);
            let mut refs = HashSet::new();
            refs.insert(c);
            self.child_normal.insert(coord, (expr.to_string(), refs));

            // Check for cycles
            if self.has_cycle_from(coord) {
                // Cycle detected - remove the dependency we just added
                self.parents_normal.entry(c).or_default().remove(&coord);
                self.child_normal.remove(&coord);
                // Restore old child dependencies
                if let Some(old_normal) = old_child_normal {
                    self.child_normal.insert(coord, old_normal);
                }
                if let Some(old_range) = old_child_range {
                    self.child_range.insert(coord, old_range);
                }
                // Restore parents
                for (parent_coord, child_coord) in removed_from_parents {
                    self.parents_normal
                        .entry(parent_coord)
                        .or_default()
                        .insert(child_coord);
                }
                // Keep the old cell value
                self.cells[coord.1 as usize][coord.0 as usize] = old_cell_value;
                return 4;
            }
            // No cycle, proceed with evaluation
            let v = self.get_val(c);
            match v {
                Some(val) => self.cells[coord.1 as usize][coord.0 as usize] = Cell::Value(val),
                None => self.cells[coord.1 as usize][coord.0 as usize] = Cell::Err,
            }
            self.recalc_dependents(coord);
            return 0;
        }

        // 2d) Literal: "42"
        if let Ok(v) = expr.parse::<i32>() {
            self.cells[coord.1 as usize][coord.0 as usize] = Cell::Value(v);
            self.recalc_dependents(coord);
            return 0;
        }

        // 2e) Anything else → abort with no change
        // RESTORE OLD CHILD DEPENDENCIES
        if let Some(old_normal) = old_child_normal {
            self.child_normal.insert(coord, old_normal);
        }
        if let Some(old_range) = old_child_range {
            self.child_range.insert(coord, old_range);
        }
        // Restore parents
        for (parent_coord, child_coord) in removed_from_parents {
            self.parents_normal
                .entry(parent_coord)
                .or_default()
                .insert(child_coord);
        }

        3 // unrecognized cmd
    }

    /// Recompute all dependents of `start`.  If division-by-zero occurs in a child,
    /// that child becomes `Err`; any other error in recomputation leaves it untouched.
    pub fn recalc_dependents(&mut self, start: (u16, u16)) {
        // Keep track of all cells that need to be recalculated
        let mut all_cells_to_update = Vec::new();
        let mut visited = HashSet::new();

        // Collect all cells affected by the change, including indirect dependencies
        let mut queue = vec![start];
        while let Some(cell) = queue.pop() {
            if !visited.insert(cell) {
                continue; // Skip if already visited
            }

            // Add to cells that need updating
            all_cells_to_update.push(cell);

            // Add normal dependents to queue
            if let Some(dependents) = self.parents_normal.get(&cell) {
                for &dependent in dependents {
                    if !visited.contains(&dependent) {
                        queue.push(dependent);
                    }
                }
            }

            // Check for range dependencies
            for (&range_cell, (_, range_start, range_end)) in &self.child_range {
                if is_within_range(cell, *range_start, *range_end) && !visited.contains(&range_cell)
                {
                    queue.push(range_cell);
                }
            }
        }

        // Now sort these cells topologically for correct calculation order
        let mut visited = HashSet::new();
        let mut visiting = HashSet::new();
        let mut topo_order = Vec::new();

        // Helper function to perform topological sort using DFS
        fn dfs(
            cell: (u16, u16),
            parents_normal: &HashMap<(u16, u16), HashSet<(u16, u16)>>,
            child_range: &HashMap<(u16, u16), ChildRangeType>,
            visited: &mut HashSet<(u16, u16)>,
            visiting: &mut HashSet<(u16, u16)>,
            topo_order: &mut Vec<(u16, u16)>,
        ) {
            if visited.contains(&cell) {
                return;
            }

            // Check for circular dependencies
            if !visiting.insert(cell) {
                // We've detected a cycle, just return without adding to topo_order
                return;
            }

            // Visit all dependents of this cell
            if let Some(dependents) = parents_normal.get(&cell) {
                for &dependent in dependents {
                    dfs(
                        dependent,
                        parents_normal,
                        child_range,
                        visited,
                        visiting,
                        topo_order,
                    );
                }
            }

            // Check range dependencies
            for (&range_cell, (_, range_start, range_end)) in child_range {
                if is_within_range(cell, *range_start, *range_end) && !visited.contains(&range_cell)
                {
                    dfs(
                        range_cell,
                        parents_normal,
                        child_range,
                        visited,
                        visiting,
                        topo_order,
                    );
                }
            }

            // Add current cell to result after all its dependents
            visiting.remove(&cell);
            visited.insert(cell);
            topo_order.push(cell);
        }

        // Build topological ordering from all affected cells
        for &cell in &all_cells_to_update {
            if !visited.contains(&cell) {
                dfs(
                    cell,
                    &self.parents_normal,
                    &self.child_range,
                    &mut visited,
                    &mut visiting,
                    &mut topo_order,
                );
            }
        }

        // Process cells in reverse topological order (dependencies before dependents)
        for cur in topo_order.iter().rev() {
            // Skip the start cell if it was already updated (e.g., by a set_cell call)
            if *cur == start {
                //this change fixed the issue of sleep (earlier it was *cur == start && all_cells_to_update.len() > 1)
                continue;
            }

            // compute new value for `cur`
            let new_cell = if let Some((formula, _)) = self.child_normal.get(cur).cloned() {
                // SLEEP function handling
                if formula.starts_with("SLEEP(") && formula.ends_with(")") {
                    let arg_str = &formula[6..formula.len() - 1];

                    // Try to parse as a literal integer
                    if let Ok(sleep_time) = arg_str.parse::<i32>() {
                        // Direct sleep with constant
                        if sleep_time > 0 {
                            thread::sleep(Duration::from_secs(sleep_time as u64));
                        }
                        Cell::Value(sleep_time)
                    }
                    // Try to parse as a cell reference
                    else if let Some(ref_cell) = MyParser::cell_name_to_coord(arg_str) {
                        match self.get_val(ref_cell) {
                            Some(sleep_time) => {
                                // Sleep using the referenced cell's value
                                if sleep_time > 0 {
                                    thread::sleep(Duration::from_secs(sleep_time as u64));
                                    //this is the issue.
                                }
                                Cell::Value(sleep_time)
                            }
                            None => Cell::Err,
                        }
                    } else {
                        Cell::Err
                    }
                }
                // binary?
                else if let Some((op_char, lhs_s, rhs_s)) = MyParser::split_binary(&formula) {
                    let op_code = match op_char {
                        '+' => 1,
                        '-' => 2,
                        '*' => 3,
                        '/' => 5,
                        _ => continue, // shouldn't happen
                    };

                    let a = if let Some(c) = MyParser::cell_name_to_coord(lhs_s) {
                        self.get_val(c)
                    } else {
                        lhs_s.parse::<i32>().ok()
                    };
                    let b = if let Some(c) = MyParser::cell_name_to_coord(rhs_s) {
                        self.get_val(c)
                    } else {
                        rhs_s.parse::<i32>().ok()
                    };

                    if op_code == 5 && b == Some(0) {
                        Cell::Err
                    } else if let (Some(a_val), Some(b_val)) = (a, b) {
                        if let Some(v) = eval_binary(op_code, a_val, b_val) {
                            Cell::Value(v)
                        } else {
                            Cell::Err
                        }
                    } else {
                        Cell::Err
                    }
                }
                // single‐cell ref?
                else if let Some(c) = MyParser::cell_name_to_coord(&formula) {
                    match self.get_val(c) {
                        Some(val) => Cell::Value(val),
                        None => Cell::Err,
                    }
                }
                // literal?
                else if let Ok(v) = formula.parse::<i32>() {
                    Cell::Value(v)
                } else {
                    continue;
                }
            }
            // range?
            else if let Some((formula, range_start, range_end)) =
                self.child_range.get(cur).cloned()
            {
                let func = &formula[..formula.find('(').unwrap_or(0)];
                if let Some(v) = eval_range(func, range_start, range_end, |c| self.get_val(c)) {
                    Cell::Value(v)
                } else {
                    Cell::Err
                }
            } else {
                continue;
            };

            self.cells[cur.1 as usize][cur.0 as usize] = new_cell;
        }
    }

    pub fn display_to<W: std::io::Write>(
        &self,
        writer: &mut W,
        start_row: usize,
        start_col: usize,
        max_rows: usize,
        max_cols: usize,
    ) -> std::io::Result<()> {
        write!(writer, "    ")?;
        for c in (start_col + 1)..=(start_col + max_cols).min(self.cols) {
            write!(writer, "{:>8}", col_to_letter(c))?;
        }
        writeln!(writer)?;

        for r in (start_row + 1)..=(start_row + max_rows).min(self.rows) {
            write!(writer, "{:>3} ", r)?;
            for c in (start_col + 1)..=(start_col + max_cols).min(self.cols) {
                match &self.cells[r][c] {
                    Cell::Value(v) => write!(writer, "{:>8}", v)?,
                    Cell::Err => write!(writer, "{:>8}", "ERR")?,
                }
            }
            writeln!(writer)?;
        }
        Ok(())
    }

    // Keep the original display function for backward compatibility
    pub fn display(&self, start_row: usize, start_col: usize, max_rows: usize, max_cols: usize) {
        self.display_to(
            &mut std::io::stdout(),
            start_row,
            start_col,
            max_rows,
            max_cols,
        )
        .expect("Failed to write to stdout");
    }

    pub fn has_cycle_from(&self, start_cell: (u16, u16)) -> bool {
        let mut visited = HashSet::new();
        let mut path = HashSet::new();

        // Simply check if there's a cycle reachable from the start cell
        self.is_cyclic(start_cell, &mut visited, &mut path)
    }

    // Helper function for cycle detection using DFS
    fn is_cyclic(
        &self,
        cell: (u16, u16),
        visited: &mut HashSet<(u16, u16)>,
        path: &mut HashSet<(u16, u16)>,
    ) -> bool {
        visited.insert(cell);
        path.insert(cell);

        // Check normal dependencies
        if let Some(refs) = self.child_normal.get(&cell) {
            for &ref_cell in &refs.1 {
                if !visited.contains(&ref_cell) {
                    if self.is_cyclic(ref_cell, visited, path) {
                        return true;
                    }
                } else if path.contains(&ref_cell) {
                    // Found a cycle
                    return true;
                }
            }
        }

        // Check range dependencies
        if let Some((_, start, end)) = self.child_range.get(&cell) {
            for col in start.0..=end.0 {
                for row in start.1..=end.1 {
                    let ref_cell = (col, row);
                    if !visited.contains(&ref_cell) {
                        if self.is_cyclic(ref_cell, visited, path) {
                            return true;
                        }
                    } else if path.contains(&ref_cell) {
                        // Found a cycle
                        return true;
                    }
                }
            }
        }

        // Remove cell from current path as we backtrack
        path.remove(&cell);
        false
    }
}

// Helper function to check if a cell is within a range
fn is_within_range(cell: (u16, u16), start: (u16, u16), end: (u16, u16)) -> bool {
    let (col, row) = cell;
    let (start_col, start_row) = start;
    let (end_col, end_row) = end;

    // Make sure we have a properly ordered range
    let min_col = start_col.min(end_col);
    let max_col = start_col.max(end_col);
    let min_row = start_row.min(end_row);
    let max_row = start_row.max(end_row);

    // Check if the cell is within the range bounds
    col >= min_col && col <= max_col && row >= min_row && row <= max_row
}

#[cfg(test)]
mod tests {
    use super::*;
    // Assuming myparser module is in your crate

    #[test]
    fn test_col_to_letter() {
        assert_eq!(col_to_letter(1), "A");
        assert_eq!(col_to_letter(26), "Z");
        assert_eq!(col_to_letter(27), "AA");
        assert_eq!(col_to_letter(52), "AZ");
        assert_eq!(col_to_letter(53), "BA");
        assert_eq!(col_to_letter(702), "ZZ");
        assert_eq!(col_to_letter(703), "AAA");
    }

    #[test]
    fn test_set_literal_value() {
        let mut sheet = Spreadsheet::new(10, 10);

        // Test setting a literal value
        let result = sheet.set_cell((1, 1), "42");
        assert_eq!(result, 0); // Success
        assert_eq!(sheet.get_val((1, 1)), Some(42));

        // Test setting a negative value
        let result = sheet.set_cell((2, 2), "-123");
        assert_eq!(result, 0); // Success
        assert_eq!(sheet.get_val((2, 2)), Some(-123));
    }

    #[test]
    fn test_set_invalid_cell() {
        let mut sheet = Spreadsheet::new(10, 10);

        // Test setting a value to a cell outside the bounds
        let result = sheet.set_cell((11, 11), "42");
        assert_eq!(result, 1); // Invalid cell code

        // Cells within bounds should be settable
        let result = sheet.set_cell((10, 10), "42");
        assert_eq!(result, 0); // Success
    }

    #[test]
    fn test_set_cell_reference() {
        let mut sheet = Spreadsheet::new(10, 10);

        // Set a literal value in A1
        sheet.set_cell((1, 1), "42");

        // Set B2 to reference A1
        let result = sheet.set_cell((2, 2), "A1");
        assert_eq!(result, 0); // Success
        assert_eq!(sheet.get_val((2, 2)), Some(42));

        // Verify dependency is tracked
        assert!(sheet.parents_normal.contains_key(&(1, 1)));
        assert!(sheet.parents_normal.get(&(1, 1)).unwrap().contains(&(2, 2)));
        assert!(sheet.child_normal.contains_key(&(2, 2)));

        // Change A1 and verify B2 updates
        sheet.set_cell((1, 1), "99");
        assert_eq!(sheet.get_val((1, 1)), Some(99));
        assert_eq!(sheet.get_val((2, 2)), Some(99));
    }

    #[test]
    fn test_set_binary_formula() {
        let mut sheet = Spreadsheet::new(10, 10);

        // Set some values
        sheet.set_cell((1, 1), "5");
        sheet.set_cell((2, 2), "10");

        // Test addition
        let result = sheet.set_cell((3, 3), "A1+B2");
        assert_eq!(result, 0); // Success
        assert_eq!(sheet.get_val((3, 3)), Some(15));

        // Test subtraction
        let result = sheet.set_cell((4, 4), "B2-A1");
        assert_eq!(result, 0); // Success
        assert_eq!(sheet.get_val((4, 4)), Some(5));

        // Test multiplication
        let result = sheet.set_cell((5, 5), "A1*B2");
        assert_eq!(result, 0); // Success
        assert_eq!(sheet.get_val((5, 5)), Some(50));

        // Test division
        let result = sheet.set_cell((6, 6), "B2/A1");
        assert_eq!(result, 0); // Success
        assert_eq!(sheet.get_val((6, 6)), Some(2));

        // Test division by zero
        let result = sheet.set_cell((7, 7), "A1/0");
        assert_eq!(result, 0); // It should still succeed but cell value is ERR
        assert_eq!(sheet.get_val((7, 7)), None);

        // Test with literal and cell reference
        let result = sheet.set_cell((8, 8), "A1+20");
        assert_eq!(result, 0); // Success
        assert_eq!(sheet.get_val((8, 8)), Some(25));
    }

    #[test]
    fn test_range_functions() {
        let mut sheet = Spreadsheet::new(10, 10);

        // Set some values
        sheet.set_cell((1, 1), "10");
        sheet.set_cell((2, 1), "20");
        sheet.set_cell((3, 1), "30");
        sheet.set_cell((1, 2), "40");
        sheet.set_cell((2, 2), "50");
        sheet.set_cell((3, 2), "60");

        // Test SUM
        let result = sheet.set_cell((5, 5), "SUM(A1:C2)");
        assert_eq!(result, 0); // Success
        assert_eq!(sheet.get_val((5, 5)), Some(210)); // 10+20+30+40+50+60

        // Test AVERAGE
        let result = sheet.set_cell((6, 6), "AVG(A1:C2)");
        assert_eq!(result, 0); // Success
        assert_eq!(sheet.get_val((6, 6)), Some(35)); // (10+20+30+40+50+60)/6

        // Test MIN
        let result = sheet.set_cell((7, 7), "MIN(A1:C2)");
        assert_eq!(result, 0); // Success
        assert_eq!(sheet.get_val((7, 7)), Some(10));

        // Test MAX
        let result = sheet.set_cell((8, 8), "MAX(A1:C2)");
        assert_eq!(result, 0); // Success
        assert_eq!(sheet.get_val((8, 8)), Some(60));

        // Test invalid range (start > end)
        let result = sheet.set_cell((9, 9), "SUM(C2:A1)");
        assert_eq!(result, 3); // Invalid command code
    }

    #[test]
    fn test_sleep_function() {
        let mut sheet = Spreadsheet::new(10, 10);

        // Set a literal sleep time (0 to avoid actually sleeping in tests)
        let result = sheet.set_cell((1, 1), "SLEEP(-1)");
        assert_eq!(result, 0); // Success
        assert_eq!(sheet.get_val((1, 1)), Some(-1));

        // Set a cell with a value
        sheet.set_cell((2, 2), "1");

        // Set a sleep with cell reference
        let result = sheet.set_cell((3, 3), "SLEEP(B2)");
        assert_eq!(result, 0); // Success
        assert_eq!(sheet.get_val((3, 3)), Some(1));

        // Test with invalid range in SLEEP
        let result = sheet.set_cell((4, 4), "SLEEP(A1:B2)");
        assert_eq!(result, 3); // Invalid command code
    }

    #[test]
    fn test_cycle_detection() {
        let mut sheet = Spreadsheet::new(10, 10);

        // Create a valid dependency chain: A1 -> B2 -> C3
        sheet.set_cell((1, 1), "42");
        sheet.set_cell((2, 2), "A1");
        sheet.set_cell((3, 3), "B2");

        // Try to create a cycle: C3 -> A1 (would make A1 -> B2 -> C3 -> A1)
        let result = sheet.set_cell((1, 1), "C3");
        assert_eq!(result, 4); // Cycle detected code

        // A1 should still have its original value
        assert_eq!(sheet.get_val((1, 1)), Some(42));

        // Test direct self-reference
        let result = sheet.set_cell((4, 4), "D4");
        assert_eq!(result, 4); // Cycle detected code

        // Test cycle with range function
        sheet.set_cell((5, 5), "10");
        let result = sheet.set_cell((5, 5), "SUM(E5:E5)");
        assert_eq!(result, 4); // Cycle detected code
    }

    #[test]
    fn test_dependency_recalculation() {
        let mut sheet = Spreadsheet::new(10, 10);

        // Setup a chain of dependencies
        sheet.set_cell((1, 1), "5"); // A1 = 5
        sheet.set_cell((2, 2), "A1*2"); // B2 = A1*2 = 10
        sheet.set_cell((3, 3), "B2+3"); // C3 = B2+3 = 13

        // Verify initial values
        assert_eq!(sheet.get_val((1, 1)), Some(5));
        assert_eq!(sheet.get_val((2, 2)), Some(10));
        assert_eq!(sheet.get_val((3, 3)), Some(13));

        // Change A1 and verify chain updates
        sheet.set_cell((1, 1), "7");
        assert_eq!(sheet.get_val((1, 1)), Some(7));
        assert_eq!(sheet.get_val((2, 2)), Some(14)); // B2 = 7*2 = 14
        assert_eq!(sheet.get_val((3, 3)), Some(17)); // C3 = 14+3 = 17
    }

    #[test]
    fn test_range_dependency_updates() {
        let mut sheet = Spreadsheet::new(10, 10);

        // Set some values in a range
        sheet.set_cell((1, 1), "10");
        sheet.set_cell((2, 1), "20");
        sheet.set_cell((1, 2), "30");
        sheet.set_cell((2, 2), "40");

        // Set a cell using range function
        sheet.set_cell((5, 5), "SUM(A1:B2)");
        assert_eq!(sheet.get_val((5, 5)), Some(100)); // 10+20+30+40

        // Change a value in the range and verify sum updates
        sheet.set_cell((1, 1), "15");
        assert_eq!(sheet.get_val((5, 5)), Some(105)); // 15+20+30+40
    }

    #[test]
    fn test_error_propagation() {
        let mut sheet = Spreadsheet::new(10, 10);

        // Create a division by zero error
        sheet.set_cell((1, 1), "10/0");
        assert_eq!(sheet.get_val((1, 1)), None); // None indicates Cell::Err

        // Reference the error cell
        sheet.set_cell((2, 2), "A1");
        assert_eq!(sheet.get_val((2, 2)), None); // Error should propagate

        // Use the error cell in a formula
        sheet.set_cell((3, 3), "A1+5");
        assert_eq!(sheet.get_val((3, 3)), None); // Error should propagate

        // Use the error cell in a range function
        sheet.set_cell((4, 4), "SUM(A1:A1)");
        assert_eq!(sheet.get_val((4, 4)), None); // Error should propagate
    }

    /*    #[test]
    fn test_invalid_formulas() {
        let mut sheet = Spreadsheet::new(10, 10);

        // Test invalid operator
        let result = sheet.set_cell((1, 1), "10%5");
        assert_eq!(result, 3); // Invalid command code

        // Test invalid range function
        let result = sheet.set_cell((2, 2), "INVALID(A1:B2)");
        assert_eq!(result, 3); // Invalid command code

        // Test formula with syntax error
        let result = sheet.set_cell((3, 3), "A1++B2");
        assert_eq!(result, 3); // Invalid command code
    }*/

    #[test]
    fn test_mixed_references_and_literals() {
        let mut sheet = Spreadsheet::new(10, 10);

        sheet.set_cell((1, 1), "10");

        // Test with cell reference and literal
        let result = sheet.set_cell((2, 2), "A1+5");
        assert_eq!(result, 0);
        assert_eq!(sheet.get_val((2, 2)), Some(15));

        // Test with literal and cell reference
        let result = sheet.set_cell((3, 3), "5+A1");
        assert_eq!(result, 0);
        assert_eq!(sheet.get_val((3, 3)), Some(15));

        // Test with cell reference and cell reference
        sheet.set_cell((4, 4), "20");
        let result = sheet.set_cell((5, 5), "A1+D4");
        assert_eq!(result, 0);
        assert_eq!(sheet.get_val((5, 5)), Some(30));
    }

    #[test]
    fn test_clear_dependencies() {
        let mut sheet = Spreadsheet::new(10, 10);

        // Setup dependencies
        sheet.set_cell((1, 1), "10");
        sheet.set_cell((2, 2), "A1");

        // Verify dependency exists
        assert!(sheet.parents_normal.contains_key(&(1, 1)));
        assert!(sheet.parents_normal.get(&(1, 1)).unwrap().contains(&(2, 2)));
        assert!(sheet.child_normal.contains_key(&(2, 2)));

        // Change B2 to a literal
        sheet.set_cell((2, 2), "20");

        // Verify dependency is removed
        assert!(!sheet.parents_normal.get(&(1, 1)).unwrap().contains(&(2, 2)));
        let a = sheet.child_normal.get(&(2, 2));
        assert!(a.is_none());
    }

    #[test]
    fn test_is_within_range() {
        // Test cell inside range
        assert!(is_within_range((2, 2), (1, 1), (3, 3)));

        // Test cell on the edge of range
        assert!(is_within_range((1, 1), (1, 1), (3, 3)));
        assert!(is_within_range((3, 3), (1, 1), (3, 3)));

        // Test cell outside range
        assert!(!is_within_range((4, 4), (1, 1), (3, 3)));

        // Test with reversed range coordinates
        assert!(is_within_range((2, 2), (3, 3), (1, 1)));
    }
}

#[test]
fn test_sleep_function_with_cell_reference_error() {
    let mut sheet = Spreadsheet::new(10, 10);

    // Set a cell that will be Err
    sheet.set_cell((1, 1), "10/0");
    assert_eq!(sheet.get_val((1, 1)), None); // Should be Err

    // Test SLEEP with reference to an Err cell
    let result = sheet.set_cell((2, 2), "SLEEP(A1)");
    assert_eq!(result, 0); // Should succeed but result in Err
    assert_eq!(sheet.get_val((2, 2)), None); // Should be Err

    // Test with invalid cell reference
    let result = sheet.set_cell((3, 3), "SLEEP(Z99)"); // Out of bounds reference
    assert_eq!(result, 0);
    assert_eq!(sheet.get_val((3, 3)), None); // Should be Err
}

#[test]
fn test_binary_operations_with_errors() {
    let mut sheet = Spreadsheet::new(10, 10);

    // Set a cell with Err
    sheet.set_cell((1, 1), "10/0");

    // Test binary operation with an Err cell
    let result = sheet.set_cell((2, 2), "A1+5");
    assert_eq!(result, 0);
    assert_eq!(sheet.get_val((2, 2)), None); // Should be Err

    // Test with two Err cells
    sheet.set_cell((3, 3), "10/0");
    let result = sheet.set_cell((4, 4), "A1+C3");
    assert_eq!(result, 0);
    assert_eq!(sheet.get_val((4, 4)), None); // Should be Err

    // Test with invalid cell reference
    let result = sheet.set_cell((5, 5), "Z99+5");
    assert_eq!(result, 0);
    assert_eq!(sheet.get_val((5, 5)), None); // Should be Err due to invalid reference
}

#[test]
fn test_binary_operations_parsable_expressions() {
    let mut sheet = Spreadsheet::new(10, 10);

    // Test with invalid left-hand side
    let result = sheet.set_cell((1, 1), "abc+5");
    assert_eq!(result, 3); // Should return unrecognized command

    // Test with invalid right-hand side
    let result = sheet.set_cell((2, 2), "5+xyz");
    assert_eq!(result, 3); // Should return unrecognized command

    // Test with invalid operator
    let result = sheet.set_cell((3, 3), "5%10");
    assert_eq!(result, 3); // Should return unrecognized command
}

#[test]
fn test_formula_cycle_detection_with_sleep() {
    let mut sheet = Spreadsheet::new(10, 10);

    // Setup reference chain with sleep
    sheet.set_cell((1, 1), "5");
    sheet.set_cell((2, 2), "SLEEP(A1)");

    // Try to create a cycle
    let result = sheet.set_cell((1, 1), "B2");
    assert_eq!(result, 4); // Should detect cycle
    assert_eq!(sheet.get_val((1, 1)), Some(5)); // Should retain original value
}

#[test]
fn test_recalculation_with_sleep() {
    let mut sheet = Spreadsheet::new(10, 10);

    // Setup chain with sleep
    sheet.set_cell((1, 1), "1");
    sheet.set_cell((2, 2), "SLEEP(A1)");
    sheet.set_cell((3, 3), "B2+1");

    // Initial values
    assert_eq!(sheet.get_val((1, 1)), Some(1));
    assert_eq!(sheet.get_val((2, 2)), Some(1));
    assert_eq!(sheet.get_val((3, 3)), Some(2));

    // Change the sleep time
    sheet.set_cell((1, 1), "0"); // Set to 0 to avoid actual sleeping in tests

    // Check recalculation
    assert_eq!(sheet.get_val((1, 1)), Some(0));
    assert_eq!(sheet.get_val((2, 2)), Some(0));
    assert_eq!(sheet.get_val((3, 3)), Some(1));
}

#[test]
fn test_multiple_binary_operation_chains() {
    let mut sheet = Spreadsheet::new(10, 10);

    // Setup complex calculation chain
    sheet.set_cell((1, 1), "10");
    sheet.set_cell((2, 2), "A1*2"); // B2 = 20
    sheet.set_cell((3, 3), "B2-5"); // C3 = 15
    sheet.set_cell((4, 4), "C3/3"); // D4 = 5
    sheet.set_cell((5, 5), "D4+A1"); // E5 = 15

    // Check initial values
    assert_eq!(sheet.get_val((2, 2)), Some(20));
    assert_eq!(sheet.get_val((3, 3)), Some(15));
    assert_eq!(sheet.get_val((4, 4)), Some(5));
    assert_eq!(sheet.get_val((5, 5)), Some(15));

    // Change base value and check recalculation
    sheet.set_cell((1, 1), "20");

    assert_eq!(sheet.get_val((1, 1)), Some(20));
    assert_eq!(sheet.get_val((2, 2)), Some(40)); // B2 = 20*2 = 40
    assert_eq!(sheet.get_val((3, 3)), Some(35)); // C3 = 40-5 = 35
    assert_eq!(sheet.get_val((4, 4)), Some(11)); // D4 = 35/3 = 11 (integer division)
    assert_eq!(sheet.get_val((5, 5)), Some(31)); // E5 = 11+20 = 31
}

#[test]
fn test_display_function() {
    let mut sheet = Spreadsheet::new(5, 5);

    // Populate with test data
    sheet.set_cell((1, 1), "10"); // A1
    sheet.set_cell((2, 2), "20"); // B2
    sheet.set_cell((3, 3), "30"); // C3
    sheet.set_cell((4, 4), "A1/0"); // D4 (Error)
    sheet.set_cell((5, 5), "50"); // E5

    // Capture output in a string buffer
    let mut output = Vec::new();
    sheet.display_to(&mut output, 0, 0, 5, 5).unwrap();
    let output_str = String::from_utf8(output).unwrap();

    // Verify expected content
    assert!(output_str.contains("A"));
    assert!(output_str.contains("B"));
    assert!(output_str.contains("10"));
    assert!(output_str.contains("20"));
    assert!(output_str.contains("ERR"));
    assert!(output_str.contains("50"));
}

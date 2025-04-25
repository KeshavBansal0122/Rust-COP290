use crate::common::cell_value::{CellError, CellValue};
use crate::common::expression::CellRange;
use crate::common::structs::AbsCell;
use crate::embedded_backend::storage::Storage;

pub fn max(storage: &Storage, cell: AbsCell, range: &CellRange) -> Result<f64, CellError> {
    let top_left = range.top_left.to_abs(cell);
    let bottom_right = range.bottom_right.to_abs(cell);

    let mut max_value = f64::MIN;
    let mut is_range_empty = true;
    for (_, val) in storage.get_value_range_sparse(top_left, bottom_right) {
        match val {
            Ok(val) => match val {
                CellValue::Number(x) => {
                    max_value = max_value.max(*x);
                    is_range_empty = false;
                }
                CellValue::String(_) => return Err(CellError::DependsOnNonNumeric),
                CellValue::Empty => {}
            },
            Err(_) => return Err(CellError::DependsOnErr),
        }
    }

    if is_range_empty {
        return Ok(0.0);
    }

    Ok(max_value)
}

pub fn min(storage: &Storage, cell: AbsCell, range: &CellRange) -> Result<f64, CellError> {
    let top_left = range.top_left.to_abs(cell);
    let bottom_right = range.bottom_right.to_abs(cell);

    let mut min_value = f64::MAX;
    let mut is_range_empty = true;
    for (_, val) in storage.get_value_range_sparse(top_left, bottom_right) {
        match val {
            Ok(val) => match val {
                CellValue::Number(x) => {
                    min_value = min_value.min(*x);
                    is_range_empty = false;
                }
                CellValue::String(_) => return Err(CellError::DependsOnNonNumeric),
                CellValue::Empty => {}
            },
            Err(_) => return Err(CellError::DependsOnErr),
        }
    }

    if is_range_empty {
        return Ok(0.0);
    }
    Ok(min_value)
}

pub fn average(storage: &Storage, cell: AbsCell, range: &CellRange) -> Result<f64, CellError> {
    let top_left = range.top_left.to_abs(cell);
    let bottom_right = range.bottom_right.to_abs(cell);

    let mut total = 0.0;
    let mut count = 0;
    for (_, val) in storage.get_value_range_sparse(top_left, bottom_right) {
        match val {
            Ok(val) => match val {
                CellValue::Number(x) => {
                    total += *x;
                    count += 1;
                }
                CellValue::String(_) => return Err(CellError::DependsOnNonNumeric),
                CellValue::Empty => {}
            },
            Err(_) => return Err(CellError::DependsOnErr),
        }
    }

    if count == 0 {
        return Ok(0.0);
    }

    Ok(total / count as f64)
}

pub fn sum(storage: &Storage, cell: AbsCell, range: &CellRange) -> Result<f64, CellError> {
    let top_left = range.top_left.to_abs(cell);
    let bottom_right = range.bottom_right.to_abs(cell);

    let mut total = 0.0;
    for (_, val) in storage.get_value_range_sparse(top_left, bottom_right) {
        match val {
            Ok(val) => match val {
                CellValue::Number(x) => total += *x,
                CellValue::String(_) => return Err(CellError::DependsOnNonNumeric),
                CellValue::Empty => {}
            },
            Err(_) => return Err(CellError::DependsOnErr),
        }
    }

    Ok(total)
}

pub fn stdev(storage: &Storage, cell: AbsCell, range: &CellRange) -> Result<f64, CellError> {
    let top_left = range.top_left.to_abs(cell);
    let bottom_right = range.bottom_right.to_abs(cell);

    let mut total = 0.0;
    let mut count = 0;
    for (_, val) in storage.get_value_range_sparse(top_left, bottom_right) {
        match val {
            Ok(val) => match val {
                CellValue::Number(x) => {
                    total += *x;
                    count += 1;
                }
                CellValue::String(_) => return Err(CellError::DependsOnNonNumeric),
                CellValue::Empty => {}
            },
            Err(_) => return Err(CellError::DependsOnErr),
        }
    }

    if count == 0 {
        return Ok(0.0);
    }

    let mean = total / count as f64;

    let mut variance = 0.0;
    for (_, val) in storage.get_value_range_sparse(top_left, bottom_right) {
        match val {
            Ok(val) => match val {
                CellValue::Number(x) => {
                    variance += (*x - mean).powi(2);
                }
                CellValue::String(_) => return Err(CellError::DependsOnNonNumeric),
                CellValue::Empty => {}
            },
            Err(_) => return Err(CellError::DependsOnErr),
        }
    }

    variance /= count as f64;

    Ok(variance.sqrt())
}

use std::thread;
use std::time::Duration;

/// Evaluate a simple binary operation.
///
/// This function performs arithmetic operations based on the operator code provided:
/// - `1` for addition.
/// - `2` for subtraction.
/// - `3` for multiplication.
/// - `5` for division.
///
/// # Arguments
/// * `op` - An integer representing the operation to perform (1, 2, 3, or 5).
/// * `a` - The left operand.
/// * `b` - The right operand.
///
/// # Returns
/// * `Some(result)` - The result of the operation if valid.
/// * `None` - If the operator is invalid or division by zero is attempted.
pub fn eval_binary(op: i8, a: i32, b: i32) -> Option<i32> {
    match op {
        1 => Some(a + b),
        2 => Some(a - b),
        3 => Some(a * b),
        5 => {
            if b == 0 {
                None
            } else {
                Some(a / b)
            }
        }
        _ => None,
    }
}

/// Calculate the minimum value in the specified range.
///
/// This function iterates over a rectangular range of cells and determines the smallest value.
/// Returns `None` if any cell in the range is in an error state.
///
/// # Arguments
/// * `start` - The top-left corner of the range as `(column, row)`.
/// * `end` - The bottom-right corner of the range as `(column, row)`.
/// * `get_val` - A callback function that returns `Some(value)` or `None` for each cell.
///
/// # Returns
/// * `Some(min_value)` - The smallest value in the range.
/// * `None` - If any cell in the range signals an error.
///
/// # Examples
/// ```rust
/// use embedded::function::min_range;
/// let get_val = |coord: (u16, u16)| Some(coord.0 as i32 + coord.1 as i32); // Example values
/// assert_eq!(min_range((1, 1), (2, 2), get_val), Some(2)); // Minimum value
/// ```
pub fn min_range<F>(start: (u16, u16), end: (u16, u16), get_val: F) -> Option<i32>
where
    F: Fn((u16, u16)) -> Option<i32>,
{
    let mut min_val = i32::MAX;
    for c in start.0..=end.0 {
        for r in start.1..=end.1 {
            let v = get_val((c, r))?;
            if v < min_val {
                min_val = v;
            }
        }
    }
    Some(min_val)
}

/// Calculate the maximum value in the specified range.
///
/// This function iterates over a rectangular range of cells and determines the largest value.
/// Returns `None` if any cell in the range is in an error state.
///
/// # Arguments
/// Same as `min_range`.
///
/// # Returns
/// * `Some(max_value)` - The largest value in the range.
/// * `None` - If any cell in the range signals an error.
///
/// # Examples
/// ```rust
/// use embedded::function::max_range;
/// let get_val = |coord: (u16, u16)| Some(coord.0 as i32 + coord.1 as i32); // Example values
/// assert_eq!(max_range((1, 1), (2, 2), get_val), Some(4)); // Maximum value
/// ```
pub fn max_range<F>(start: (u16, u16), end: (u16, u16), get_val: F) -> Option<i32>
where
    F: Fn((u16, u16)) -> Option<i32>,
{
    let mut max_val = i32::MIN;
    for c in start.0..=end.0 {
        for r in start.1..=end.1 {
            let v = get_val((c, r))?;
            if v > max_val {
                max_val = v;
            }
        }
    }
    Some(max_val)
}

/// Calculate the average (rounded down) of values in the specified range.
///
/// Computes the arithmetic mean of all values in the range.
/// Returns `None` if any cell is in an error state, or `Some(0)` if the range is empty.
///
/// # Arguments
/// Same as `min_range`.
///
/// # Returns
/// * `Some(average)` - The average of all values in the range.
/// * `None` - If any cell in the range signals an error.
///
/// # Examples
/// ```rust
/// use embedded::function::avg_range;
/// let get_val = |coord: (u16, u16)| Some(coord.0 as i32 + coord.1 as i32); // Example values
/// assert_eq!(avg_range((1, 1), (2, 2), get_val), Some(3)); // Average value
/// ```
pub fn avg_range<F>(start: (u16, u16), end: (u16, u16), get_val: F) -> Option<i32>
where
    F: Fn((u16, u16)) -> Option<i32>,
{
    let mut sum: i64 = 0;
    let mut count: i64 = 0;
    for c in start.0..=end.0 {
        for r in start.1..=end.1 {
            let v = get_val((c, r))?;
            sum += v as i64;
            count += 1;
        }
    }
    if count == 0 {
        Some(0)
    } else {
        Some((sum / count) as i32)
    }
}

/// Calculate the sum of values in the specified range.
///
/// This function iterates over a rectangular range of cells and computes the total sum.
/// Returns `None` if any cell in the range is in an error state.
///
/// # Arguments
/// Same as `min_range`.
///
/// # Returns
/// * `Some(sum)` - The total sum of all values in the range.
/// * `None` - If any cell in the range signals an error.
///
/// # Examples
/// ```rust
/// use embedded::function::sum_range;
/// let get_val = |coord: (u16, u16)| Some(coord.0 as i32 + coord.1 as i32); // Example values
/// assert_eq!(sum_range((1, 1), (2, 2), get_val), Some(12)); // Total sum
/// ```
pub fn sum_range<F>(start: (u16, u16), end: (u16, u16), get_val: F) -> Option<i32>
where
    F: Fn((u16, u16)) -> Option<i32>,
{
    let mut sum: i32 = 0;
    for c in start.0..=end.0 {
        for r in start.1..=end.1 {
            let v = get_val((c, r))?;
            sum += v;
        }
    }
    Some(sum)
}

/// Calculate the standard deviation (rounded) of values in the specified range.
///
/// This function computes the population standard deviation for the values in the specified range.
/// The result is rounded to the nearest integer. Returns `None` if any cell is in an error state or
/// if fewer than two valid cells are present.
///
/// # Arguments
/// * `start` - The top-left corner of the range as `(column, row)`.
/// * `end` - The bottom-right corner of the range as `(column, row)`.
/// * `get_val` - A callback function that returns `Some(value)` or `None` for each cell.
///
/// # Returns
/// * `Some(stdev)` - The standard deviation of the values in the range.
/// * `None` - If any cell in the range signals an error or fewer than two valid cells exist.
///
/// # Examples
/// ```rust
/// use embedded::function::stdev_range;
/// let get_val = |coord: (u16, u16)| Some(coord.0 as i32 + coord.1 as i32); // Example values
/// assert_eq!(stdev_range((1, 1), (2, 2), get_val), Some(1)); // Small range
/// assert_eq!(stdev_range((1, 1), (3, 3), get_val), Some(1)); // Larger range
/// ```
pub fn stdev_range<F>(start: (u16, u16), end: (u16, u16), get_val: F) -> Option<i32>
where
    F: Fn((u16, u16)) -> Option<i32>,
{
    // First pass: sum and count
    let mut sum: f64 = 0.0;
    let mut count: usize = 0;
    for c in start.0..=end.0 {
        for r in start.1..=end.1 {
            let v = get_val((c, r))? as f64;
            sum += v;
            count += 1;
        }
    }
    if count <= 1 {
        return Some(0);
    }
    let mean = sum / count as f64;

    // Second pass: accumulate squared deviations
    let mut var_sum: f64 = 0.0;
    for c in start.0..=end.0 {
        for r in start.1..=end.1 {
            let v = get_val((c, r))? as f64;
            let diff = v - mean;
            var_sum += diff * diff;
        }
    }
    let variance = var_sum / count as f64;
    Some(variance.sqrt().round() as i32)
}

/// Evaluate a range function (MIN/MAX/AVG/SUM/STDEV/SLEEP).
///
/// This function dispatches the specified range function (`func`) to the appropriate helper
/// method for evaluation. It supports standard aggregate functions (e.g., `MIN`, `MAX`) as well
/// as the special `SLEEP` function.
///
/// # Arguments
/// * `func` - A string representing the function name (e.g., `"SUM"` or `"SLEEP"`).
/// * `start` - The top-left corner of the range as `(column, row)`.
/// * `end` - The bottom-right corner of the range as `(column, row)`.
/// * `get_val` - A callback function that returns `Some(value)` or `None` for each cell.
///
/// # Returns
/// * `Some(result)` - The result of the range function.
/// * `None` - If the function name is invalid or any cell in the range signals an error.
///
/// # Supported Functions
/// - `"MIN"`: Calculates the minimum value in the range.
/// - `"MAX"`: Calculates the maximum value in the range.
/// - `"AVG"`: Calculates the average value in the range.
/// - `"SUM"`: Calculates the total sum of values in the range.
/// - `"STDEV"`: Calculates the standard deviation of values in the range.
/// - `"SLEEP"`: Delays execution for a specified number of seconds (the value of the first cell).
///
/// # Examples
/// ```rust
/// use embedded::function::eval_range;
/// let get_val = |coord: (u16, u16)| Some(coord.0 as i32 + coord.1 as i32); // Example values
///
/// // Evaluate range functions:
/// assert_eq!(eval_range("SUM", (1, 1), (2, 2), get_val), Some(12)); // SUM
/// assert_eq!(eval_range("AVG", (1, 1), (2, 2), get_val), Some(3)); // AVG
///
/// // Special function SLEEP:
/// let get_val_sleep = |coord: (u16, u16)| Some(2); // Simulated value for sleep
/// assert_eq!(eval_range("SLEEP", (1, 1), (1, 1), get_val_sleep), Some(2)); // SLEEP
/// ```
pub fn eval_range<F>(func: &str, start: (u16, u16), end: (u16, u16), get_val: F) -> Option<i32>
where
    F: Fn((u16, u16)) -> Option<i32>,
{
    // Special handling for SLEEP function
    if func.eq_ignore_ascii_case("SLEEP") {
        let sec = get_val(start)?;
        if sec > 0 {
            thread::sleep(Duration::from_secs(sec as u64));
        }
        return Some(sec);
    }

    // Dispatch to the appropriate helper
    match func.to_uppercase().as_str() {
        "MIN" => min_range(start, end, &get_val),
        "MAX" => max_range(start, end, &get_val),
        "AVG" => avg_range(start, end, &get_val),
        "SUM" => sum_range(start, end, &get_val),
        "STDEV" => stdev_range(start, end, &get_val),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eval_binary() {
        // Test addition
        assert_eq!(eval_binary(1, 5, 3), Some(8));
        assert_eq!(eval_binary(1, -5, 10), Some(5));

        // Test subtraction
        assert_eq!(eval_binary(2, 10, 4), Some(6));
        assert_eq!(eval_binary(2, 5, 10), Some(-5));

        // Test multiplication
        assert_eq!(eval_binary(3, 6, 7), Some(42));
        assert_eq!(eval_binary(3, -3, 4), Some(-12));

        // Test division
        assert_eq!(eval_binary(5, 10, 2), Some(5));
        assert_eq!(eval_binary(5, 7, 2), Some(3)); // Integer division rounds down
        assert_eq!(eval_binary(5, -10, 3), Some(-3)); // Integer division with negative

        // Test division by zero
        assert_eq!(eval_binary(5, 10, 0), None);

        // Test invalid operator
        assert_eq!(eval_binary(4, 10, 5), None);
        assert_eq!(eval_binary(0, 10, 5), None);
        assert_eq!(eval_binary(-1, 10, 5), None);
    }

    #[test]
    fn test_min_range() {
        // Create a mock getter function
        let values = vec![((1, 1), 10), ((1, 2), 5), ((2, 1), 15), ((2, 2), 20)];
        let get_val = |coord: (u16, u16)| -> Option<i32> {
            values.iter().find(|(c, _)| *c == coord).map(|(_, v)| *v)
        };

        // Test normal case
        assert_eq!(min_range((1, 1), (2, 2), get_val), Some(5));

        // Test single cell
        assert_eq!(min_range((1, 1), (1, 1), get_val), Some(10));

        // Test error case (missing cell)
        let get_val_with_error = |coord: (u16, u16)| -> Option<i32> {
            if coord == (2, 2) {
                None
            } else {
                get_val(coord)
            }
        };
        assert_eq!(min_range((1, 1), (2, 2), get_val_with_error), None);
    }

    #[test]
    fn test_max_range() {
        // Create a mock getter function
        let values = vec![((1, 1), 10), ((1, 2), 5), ((2, 1), 15), ((2, 2), 20)];
        let get_val = |coord: (u16, u16)| -> Option<i32> {
            values.iter().find(|(c, _)| *c == coord).map(|(_, v)| *v)
        };

        // Test normal case
        assert_eq!(max_range((1, 1), (2, 2), get_val), Some(20));

        // Test single cell
        assert_eq!(max_range((1, 1), (1, 1), get_val), Some(10));

        // Test error case (missing cell)
        let get_val_with_error = |coord: (u16, u16)| -> Option<i32> {
            if coord == (2, 2) {
                None
            } else {
                get_val(coord)
            }
        };
        assert_eq!(max_range((1, 1), (2, 2), get_val_with_error), None);
    }

    #[test]
    fn test_avg_range() {
        // Create a mock getter function
        let values = vec![((1, 1), 10), ((1, 2), 5), ((2, 1), 15), ((2, 2), 20)];
        let get_val = |coord: (u16, u16)| -> Option<i32> {
            values.iter().find(|(c, _)| *c == coord).map(|(_, v)| *v)
        };

        // Test normal case
        assert_eq!(avg_range((1, 1), (2, 2), get_val), Some(12)); // (10+5+15+20)/4 = 12.5, rounded down to 12

        // Test single cell
        assert_eq!(avg_range((1, 1), (1, 1), get_val), Some(10));

        // Test with empty range (no cells found)
        let empty_get_val = |_: (u16, u16)| -> Option<i32> { None };
        assert_eq!(avg_range((5, 5), (5, 5), empty_get_val), None);

        // Test error case (missing cell)
        let get_val_with_error = |coord: (u16, u16)| -> Option<i32> {
            if coord == (2, 2) {
                None
            } else {
                get_val(coord)
            }
        };
        assert_eq!(avg_range((1, 1), (2, 2), get_val_with_error), None);
    }

    #[test]
    fn test_sum_range() {
        // Create a mock getter function
        let values = vec![((1, 1), 10), ((1, 2), 5), ((2, 1), 15), ((2, 2), 20)];
        let get_val = |coord: (u16, u16)| -> Option<i32> {
            values.iter().find(|(c, _)| *c == coord).map(|(_, v)| *v)
        };

        // Test normal case
        assert_eq!(sum_range((1, 1), (2, 2), get_val), Some(50)); // 10+5+15+20 = 50

        // Test single cell
        assert_eq!(sum_range((1, 1), (1, 1), get_val), Some(10));

        // Test with negative values
        let neg_values = vec![((1, 1), -10), ((1, 2), 5), ((2, 1), -15), ((2, 2), 20)];
        let neg_get_val = |coord: (u16, u16)| -> Option<i32> {
            neg_values
                .iter()
                .find(|(c, _)| *c == coord)
                .map(|(_, v)| *v)
        };
        assert_eq!(sum_range((1, 1), (2, 2), neg_get_val), Some(0)); // -10+5-15+20 = 0

        // Test error case (missing cell)
        let get_val_with_error = |coord: (u16, u16)| -> Option<i32> {
            if coord == (2, 2) {
                None
            } else {
                get_val(coord)
            }
        };
        assert_eq!(sum_range((1, 1), (2, 2), get_val_with_error), None);
    }

    #[test]
    fn test_stdev_range() {
        // Create a mock getter function with values having a known standard deviation
        // Using values: 2, 4, 4, 4, 5, 5, 7, 9 => mean = 5, variance = 4, stdev = 2
        let values = vec![
            ((1, 1), 2),
            ((1, 2), 4),
            ((1, 3), 4),
            ((1, 4), 4),
            ((2, 1), 5),
            ((2, 2), 5),
            ((2, 3), 7),
            ((2, 4), 9),
        ];
        let get_val = |coord: (u16, u16)| -> Option<i32> {
            values.iter().find(|(c, _)| *c == coord).map(|(_, v)| *v)
        };

        // Test normal case
        assert_eq!(stdev_range((1, 1), (2, 4), get_val), Some(2));

        // Test with fewer than 2 cells (should return 0)
        assert_eq!(stdev_range((1, 1), (1, 1), get_val), Some(0));

        // Test error case (missing cell)
        let get_val_with_error = |coord: (u16, u16)| -> Option<i32> {
            if coord == (2, 2) {
                None
            } else {
                get_val(coord)
            }
        };
        assert_eq!(stdev_range((1, 1), (2, 4), get_val_with_error), None);
    }

    #[test]
    fn test_eval_range() {
        // Create a mock getter function
        let values = vec![((1, 1), 10), ((1, 2), 5), ((2, 1), 15), ((2, 2), 20)];
        let get_val = |coord: (u16, u16)| -> Option<i32> {
            values.iter().find(|(c, _)| *c == coord).map(|(_, v)| *v)
        };

        // Test MIN function
        assert_eq!(eval_range("MIN", (1, 1), (2, 2), get_val), Some(5));

        // Test MAX function
        assert_eq!(eval_range("MAX", (1, 1), (2, 2), get_val), Some(20));

        // Test AVG function
        assert_eq!(eval_range("AVG", (1, 1), (2, 2), get_val), Some(12));

        // Test SUM function
        assert_eq!(eval_range("SUM", (1, 1), (2, 2), get_val), Some(50));

        // Test case-insensitivity
        assert_eq!(eval_range("sum", (1, 1), (2, 2), get_val), Some(50));
        assert_eq!(eval_range("Sum", (1, 1), (2, 2), get_val), Some(50));

        // Test SLEEP function (with time=0 to avoid actual sleep)
        let sleep_value = vec![((1, 1), 0)];
        let sleep_get_val = |coord: (u16, u16)| -> Option<i32> {
            sleep_value
                .iter()
                .find(|(c, _)| *c == coord)
                .map(|(_, v)| *v)
        };
        assert_eq!(eval_range("SLEEP", (1, 1), (1, 1), sleep_get_val), Some(0));

        // Test invalid function name
        assert_eq!(eval_range("INVALID", (1, 1), (2, 2), get_val), None);
    }
}

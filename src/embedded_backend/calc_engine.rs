use crate::common::cell_value::{CellError, CellValue};
use crate::common::expression::{Expression, Operator, RangeFunction};
use crate::common::structs::AbsCell;
use crate::embedded_backend::storage::Storage;
use std::thread::sleep;
use std::time::Duration;

mod functions;

/// Evaluates the expression for a given cell.
pub fn evaluate(storage: &Storage, cell: AbsCell, expr: &Expression) -> Result<f64, CellError> {
    match expr {
        Expression::Number(x) => Ok(*x),

        Expression::Cell(c) => {
            let x = storage.get_value(c.to_abs(cell));
            match x {
                Ok(val) => match val {
                    CellValue::Number(n) => Ok(*n),
                    CellValue::Empty => Ok(0.0),
                    CellValue::String(_) => Err(CellError::DependsOnNonNumeric),
                },
                Err(e) => Err(*e),
            }
        }
        Expression::BinaryOp(exp1, op, exp2) => match op {
            Operator::Add => {
                let x = evaluate(storage, cell, exp1)?;
                let y = evaluate(storage, cell, exp2)?;
                Ok(x + y)
            }
            Operator::Subtract => {
                let x = evaluate(storage, cell, exp1)?;
                let y = evaluate(storage, cell, exp2)?;
                Ok(x - y)
            }
            Operator::Multiply => {
                let x = evaluate(storage, cell, exp1)?;
                let y = evaluate(storage, cell, exp2)?;
                Ok(x * y)
            }
            Operator::Divide => {
                let x = evaluate(storage, cell, exp1)?;
                let y = evaluate(storage, cell, exp2)?;
                if y == 0.0 {
                    Err(CellError::DivideByZero)
                } else {
                    Ok(x / y)
                }
            }
        },

        Expression::RangeFunction(f, range) => match f {
            RangeFunction::Min => functions::min(storage, cell, range),
            RangeFunction::Max => functions::max(storage, cell, range),
            RangeFunction::Avg => functions::average(storage, cell, range),
            RangeFunction::Sum => functions::sum(storage, cell, range),
            RangeFunction::Stdev => functions::stdev(storage, cell, range),
        },
        Expression::Sleep(exp) => {
            let x = evaluate(storage, cell, exp)?;
            if x > 0.0 {
                sleep(Duration::from_secs_f64(x));
            }
            Ok(x)
        }
    }
}

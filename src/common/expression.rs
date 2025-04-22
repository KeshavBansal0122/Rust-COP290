use std::fmt::{self, Display};
use crate::common::structs::{AbsCell, RelCell};

#[derive(Copy, Clone, Debug, Hash, PartialEq)]
pub enum Operator {
    Add,
    Subtract,
    Multiply,
    Divide,
}

#[derive(Copy, Clone, Debug, Hash, PartialEq)]
pub enum RangeFunction {
    Min,
    Max,
    Avg,
    Sum,
    Stdev,
}

#[derive(Clone, Debug, Hash, PartialEq)]
pub struct CellRange {
    pub top_left: RelCell,
    pub bottom_right: RelCell,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Expression {
    Number(f64),
    Cell(RelCell),
    BinaryOp(Box<Expression>, Operator, Box<Expression>),
    RangeFunction(RangeFunction, CellRange),
    Sleep(Box<Expression>),
}


impl Display for Operator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let op_str = match self {
            Operator::Add => "+",
            Operator::Subtract => "-",
            Operator::Multiply => "*",
            Operator::Divide => "/",
        };
        write!(f, "{}", op_str)
    }
}

impl Display for RangeFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let func_str = match self {
            RangeFunction::Min => "MIN",
            RangeFunction::Max => "MAX",
            RangeFunction::Avg => "AVG",
            RangeFunction::Sum => "SUM",
            RangeFunction::Stdev => "STDEV",
        };
        write!(f, "{}", func_str)
    }
}

impl CellRange {
    fn to_string(&self, cell: AbsCell) -> String {
        let tl = self.top_left.to_abs(cell);
        let br = self.bottom_right.to_abs(cell);
        format!("{}:{}", tl, br)
    }
}

impl Expression {
    fn to_string(&self, cell: AbsCell) -> String {
        match self {
            Expression::Number(n) => format!("{}", n),
            Expression::Cell(c) => format!("{}", c.to_abs(cell)),
            Expression::BinaryOp(left, op, right) => {
                format!("{} {} {}", left.to_string(cell), op, right.to_string(cell))
            }
            Expression::RangeFunction(func, range) => {
                format!("{}({})", func, range.to_string(cell))
            }
            Expression::Sleep(inner) => {
                format!("SLEEP({})", inner.to_string(cell))
            }
        }
    }
}

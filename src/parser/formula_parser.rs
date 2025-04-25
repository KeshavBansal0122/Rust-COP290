use pest::Parser;
use pest::iterators::Pair;
use pest_derive::Parser;
use std::str::FromStr;

use crate::common::expression::{CellRange, Expression, Operator, RangeFunction};
use crate::common::structs::{AbsCell, RelCell};

#[derive(Parser)]
#[grammar = "parser/formula.pest"]
struct PestFormulaParser;

#[derive(Debug, Clone)]
pub struct FormulaParser {
    max_rows: u16,
    max_cols: u16,
}

impl FormulaParser {
    pub fn new(max_rows: u16, max_cols: u16) -> Self {
        FormulaParser { max_rows, max_cols }
    }

    #[allow(clippy::result_unit_err)]
    pub fn parse(&self, formula: &str, cell: AbsCell) -> Result<Expression, ()> {
        let pairs = PestFormulaParser::parse(Rule::formula, formula).map_err(|_| ())?;

        let formula_pair = pairs.peek().unwrap();
        let expr_pairs = formula_pair.into_inner().next().unwrap();

        self.parse_expression(expr_pairs, cell)
    }

    fn parse_expression(&self, pair: Pair<Rule>, cell: AbsCell) -> Result<Expression, ()> {
        match pair.as_rule() {
            Rule::expression => {
                let mut pairs = pair.into_inner();
                let mut left = self.parse_expression(pairs.next().unwrap(), cell)?;

                while let Some(op_pair) = pairs.next() {
                    let operator = match op_pair.as_rule() {
                        Rule::add => Operator::Add,
                        Rule::subtract => Operator::Subtract,
                        Rule::multiply => Operator::Multiply,
                        Rule::divide => Operator::Divide,
                        _ => unreachable!(),
                    };

                    let right = self.parse_expression(pairs.next().unwrap(), cell)?;
                    left = Expression::BinaryOp(Box::new(left), operator, Box::new(right));
                }

                Ok(left)
            }
            Rule::factor => {
                let mut pairs = pair.into_inner();
                let mut left = self.parse_expression(pairs.next().unwrap(), cell)?;

                while let Some(op_pair) = pairs.next() {
                    let operator = match op_pair.as_rule() {
                        Rule::multiply => Operator::Multiply,
                        Rule::divide => Operator::Divide,
                        _ => unreachable!(),
                    };

                    let right = self.parse_expression(pairs.next().unwrap(), cell)?;
                    left = Expression::BinaryOp(Box::new(left), operator, Box::new(right));
                }

                Ok(left)
            }
            Rule::term => {
                let inner = pair.into_inner().next().unwrap();
                self.parse_expression(inner, cell)
            }
            Rule::number => {
                let value = pair.as_str().parse::<f64>().map_err(|_| ())?;
                Ok(Expression::Number(value))
            }
            Rule::cell_ref => {
                let cell_ref = self.parse_cell_ref(pair.as_str(), cell)?;
                Ok(Expression::Cell(cell_ref))
            }
            Rule::function => {
                let function_pair = pair.into_inner().next().unwrap();
                self.parse_expression(function_pair, cell)
            }
            Rule::range_function => {
                let mut pairs = pair.into_inner();
                let function_name = pairs.next().unwrap();
                let range_pair = pairs.next().unwrap();

                let range_function = match function_name.as_str() {
                    "MIN" => RangeFunction::Min,
                    "MAX" => RangeFunction::Max,
                    "AVG" => RangeFunction::Avg,
                    "SUM" => RangeFunction::Sum,
                    "STDEV" => RangeFunction::Stdev,
                    _ => return Err(()),
                };

                let cell_range = self.parse_cell_range(range_pair, cell)?;
                Ok(Expression::RangeFunction(range_function, cell_range))
            }
            Rule::sleep_function => {
                let expr_pair = pair.into_inner().next().unwrap();
                let expr = self.parse_expression(expr_pair, cell)?;
                Ok(Expression::Sleep(Box::new(expr)))
            }
            _ => Err(()),
        }
    }

    fn parse_cell_ref(&self, ref_str: &str, cell: AbsCell) -> Result<RelCell, ()> {
        let c = AbsCell::from_str(ref_str).map_err(|_| ())?;
        if c.row >= self.max_rows as i16 || c.col >= self.max_cols as i16 {
            Err(())
        } else {
            Ok(c.to_rel(cell))
        }
    }

    fn parse_cell_range(&self, range_pair: Pair<Rule>, cell: AbsCell) -> Result<CellRange, ()> {
        let mut pairs = range_pair.into_inner();
        let top_left_str = pairs.next().unwrap().as_str();
        let bottom_right_str = pairs.next().unwrap().as_str();

        let top_left = self.parse_cell_ref(top_left_str, cell)?;
        let bottom_right = self.parse_cell_ref(bottom_right_str, cell)?;

        // Validate that the range forms a valid rectangle
        if !(top_left.row <= bottom_right.row && top_left.col <= bottom_right.col) {
            return Err(());
        }
        Ok(CellRange {
            top_left,
            bottom_right,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::super::formula_parser::FormulaParser;
    use crate::common::expression::Expression;
    use crate::common::structs::AbsCell;

    #[test]
    fn test_basic_formula() {
        let parser = FormulaParser::new(1000, 26);
        let cell = AbsCell::new(1, 1);
        let formula = "A1 + SUM(B1:Z9) + 1.0 - .3";
        let result = parser.parse(formula, cell);
        assert!(result.is_ok(), "Failed to parse: {}", formula);
    }

    #[test]
    fn test_negative_number() {
        let parser = FormulaParser::new(1000, 26);
        let cell = AbsCell::new(1, 1);
        let formula = "-.75";
        let result = parser.parse(formula, cell);
        assert!(result.is_ok(), "Failed to parse: {}", formula);

        if let Ok(expr) = result {
            match expr {
                Expression::Number(value) => assert_eq!(value, -0.75),
                _ => panic!("Expected Number expression"),
            }
        }
    }

    #[test]
    fn test_complex_expression() {
        let parser = FormulaParser::new(1000, 26);
        let cell = AbsCell::new(1, 1);
        let formula = "A1 * 2 + (B2 - C3) / 4";
        let result = parser.parse(formula, cell);
        assert!(result.is_ok(), "Failed to parse: {}", formula);
    }

    #[test]
    fn test_invalid_range() {
        let parser = FormulaParser::new(1000, 26);
        let formula = "SUM(Z9:A1)"; // Bottom-right should be below and to the right of top-left
        let cell = AbsCell::new(1, 1);

        let result = parser.parse(formula, cell);
        assert!(result.is_err(), "Should fail with invalid range");
    }

    #[test]
    fn test_out_of_bounds() {
        let parser = FormulaParser::new(1000, 26);
        // Column AA is beyond our 26-column limit (A-Z)
        let formula = "AA1 + B2";
        let cell = AbsCell::new(1, 1);

        let result = parser.parse(formula, cell);
        assert!(result.is_err(), "Should fail with out of bounds error");
    }

    #[test]
    fn test_row_out_of_bounds() {
        let parser = FormulaParser::new(1000, 26);
        // Row 1001 is beyond our 1000-row limit
        let formula = "A1001 + B2";
        let cell = AbsCell::new(1, 1);

        let result = parser.parse(formula, cell);
        assert!(result.is_err(), "Should fail with out of bounds error");
    }
}

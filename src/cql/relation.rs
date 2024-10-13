use std::error::Error;
use crate::my_errors::MyError;

const EQUAL: i32 = 0;
const GREATER_THAN: i32 = 1;
const LESS_THAN: i32 = 2;
const NOT_NUMBER: i32 = 3;

#[derive(Debug, PartialEq)]
pub struct Column<'a> {
    pub col: &'a str,
}

#[derive(Debug, PartialEq)]
pub struct Value<'a> {
    pub val: &'a str,
}

#[derive(Debug, PartialEq)]
pub enum Relation<'a> {
    Equal(&'a str, &'a str),
    NotEqual(&'a str, &'a str),
    LessThan(&'a str, &'a str),
    LessOrEqualThan(&'a str, &'a str),
    GreaterThan(&'a str, &'a str),
    GreaterOrEqualThan(&'a str, &'a str),
}

impl<'a> Relation<'a> {
    
    fn col(& self) -> &'a str {
        match self {
            Relation::Equal(col, _) 
            | Relation::NotEqual(col, _)
            | Relation::GreaterThan(col, _) 
            | Relation::GreaterOrEqualThan(col, _)
            | Relation::LessThan(col, _) 
            | Relation::LessOrEqualThan(col, _)
            => col
        }
    }

    pub fn relate(& self, cols: &Vec<str>, vals: &Vec<str>) -> bool {
        if let Some(index) = cols.iter().position(|c| c == self.col()) {
            if let Some(v) = vals.get(index) {
                match self {
                    Self::Equal(_, val) => v == *val,
                    Self::NotEqual(_, val) => v != *val,
                    Self::LessThan(_, val) => Self::compare_numbers(v, val) == LESS_THAN,
                    Self::LessOrEqualThan(_, val) => {
                        let comp = Self::compare_numbers(v, val);
                        comp == LESS_THAN || comp == EQUAL
                    },
                    Self::GreaterThan(_, val) => Self::compare_numbers(v, val) == GREATER_THAN,
                    Self::GreaterOrEqualThan(_, val) => {
                        let comp = Self::compare_numbers(v, val);
                        comp == GREATER_THAN || comp == EQUAL
                    },
                }
            } else {
                false
            }
        } else {
            false
        }
    }

    fn relate_numbers(a: &str, b: &str) -> i32 {
        match (a.parse::<f64>(), b.parse::<f64>()) {
            (Ok(number_a), Ok(number_b)) => {
                if number_a == number_b {
                    EQUAL
                } else if number_a > number_b {
                    GREATER_THAN
                } else {
                    LESS_THAN
                }
            }
            _ => NOT_NUMBER
        }
    }
}
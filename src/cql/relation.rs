use crate::config::{EQUAL, GREATER_THAN, LESS_THAN, NOT_NUMBER};

#[derive(Debug, PartialEq, Eq)]
pub struct Column {
    pub col: String,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Value {
    pub val: String,
}

#[derive(Debug, PartialEq, Eq, Clone)]
/// Aca se hacen las comparaciones de las condiciones WHERE de la consulta
/// 'Relation': Representa la operacion en su nivel atomico, ie: 'val1 op val2'
pub enum Relation {
    Equal(String, String),
    NotEqual(String, String),
    LessThan(String, String),
    LessOrEqualThan(String, String),
    GreaterThan(String, String),
    GreaterOrEqualThan(String, String),
}

impl Relation {
    fn col(&self) -> String {
        match self {
            Relation::Equal(col, _)
            | Relation::NotEqual(col, _)
            | Relation::GreaterThan(col, _)
            | Relation::GreaterOrEqualThan(col, _)
            | Relation::LessThan(col, _)
            | Relation::LessOrEqualThan(col, _) => col.to_string(),
        }
    }

    pub fn relate(&self, cols: &[String], vals: &[String]) -> bool {
        if let Some(index) = cols.iter().position(|c| *c == self.col()) {
            if let Some(v) = vals.get(index) {
                match self {
                    Self::Equal(_, val) => v == val,
                    Self::NotEqual(_, val) => v != val,
                    Self::LessThan(_, val) => Self::relate_numbers(v, val) == LESS_THAN,
                    Self::LessOrEqualThan(_, val) => {
                        let comp = Self::relate_numbers(v, val);
                        comp == LESS_THAN || comp == EQUAL
                    }
                    Self::GreaterThan(_, val) => Self::relate_numbers(v, val) == GREATER_THAN,
                    Self::GreaterOrEqualThan(_, val) => {
                        let comp = Self::relate_numbers(v, val);
                        comp == GREATER_THAN || comp == EQUAL
                    }
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
            _ => NOT_NUMBER,
        }
    }
}

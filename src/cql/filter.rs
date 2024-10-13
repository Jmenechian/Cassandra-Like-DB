use std::error::Error;
use crate::cql::my_errors::MyError;
use crate::cql::where_clause::WhereClause;

pub struct Filter<'a> {
    pub columns: &'a Vec<str>,
    pub expr: &'a WhereClause<'a>,
}

impl<'a> Filter<'a> {
    pub fn new(cols: &'a Vec<str>, expr: &'a WhereClause) ->Result<Self, Box<dyn Error>> {
        if expr.check(cols)? {
            Ok(Filter{columns: cols, expr: expr})
        } else {
            return Err(Box::new(MyError::Error("Invalid <where_clause>")))
        }
    }

    pub fn filter(& self, vals: &Vec<str>) -> bool {
        self.expr.relate(self.columns, vals)
    }
}
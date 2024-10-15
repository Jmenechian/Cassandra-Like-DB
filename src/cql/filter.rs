use crate::cql::where_clause::WhereClause;
use std::error::Error;

pub struct Filter {
    pub columns: Vec<String>,
    pub expr: WhereClause,
}

impl Filter {
    pub fn new(cols: Vec<String>, expr: WhereClause) -> Result<Self, Box<dyn Error>> {
        Ok(Filter {
            columns: cols,
            expr,
        })
    }

    pub fn filter(&self, vals: Vec<String>) -> bool {
        self.expr.relate(&self.columns, &vals)
    }
}

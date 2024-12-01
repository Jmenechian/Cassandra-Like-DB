use crate::cql::where_clause::WhereClause;
use crate::errors::my_errors::MyError;

/// Filtra las filas de una tabla de acuerdo a las conduciones del WHERE.
pub struct Filter {
    pub columns: Vec<String>,
    pub expr: WhereClause,
}

impl Filter {
    pub fn new(cols: Vec<String>, expr: WhereClause) -> Result<Self, MyError> {
        Ok(Filter {
            columns: cols,
            expr,
        })
    }

    /// Indica si una fila se tiene que filtrar o no.
    pub fn filter(&self, vals: &Vec<String>) -> bool {
        self.expr.relate(&self.columns, vals)
    }
}

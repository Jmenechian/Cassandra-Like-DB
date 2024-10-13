use std::error::Error;
use crate::cql::my_errors::MyError;
use crate::cql::relation::Relation;

#[derive(Debug, PartialEq)]
pub enum WhereClause<'a> {
    True,
    Relation(Relation<'a>),
    And(Box<WhereClause<'a>>, Box<WhereClause<'a>>),
    Or(Box<WhereClause<'a>>, Box<WhereClause<'a>>),
}

impl<'a> WhereClause<'a> {
    pub fn new(input: &str) -> Result<Self, Box<dyn Error>> {
        let mut lexer: Lexer = Lexer::new(input);
        let tokens: Vec<Token> = lexer.tokenize()?;
        let parser: Parser::new(&tokens);
        parser.parse_whereclause()?
    }
    
    pub fn relate(& self, cols: &Vec<str>, vals: &Vec<str>) -> bool {
        match self {
            Self::True => true,
            Self::Comparison(comp) => comp.relate(cols, vals),
            Self::And(expr1, expr2) => expr1.relate(cols, vals) && expr2.relate(cols, vals),
            Self::Or(expr1, expr2) => expr1.relate(cols, vals) || expr2.relate(cols, vals),
        }
    }
}
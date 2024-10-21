// Representa en su totalidad a la condicion WHERE de la consulta
// 'WhereClause': Utiliza recursion para representar la condicion WHERE

use crate::cql::lexer::*;
use crate::cql::parser::*;
use crate::cql::relation::Relation;
use crate::my_errors::MyError;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum WhereClause {
    True,
    Relation(Relation),
    And(Box<WhereClause>, Box<WhereClause>),
    Or(Box<WhereClause>, Box<WhereClause>),
}

impl WhereClause {
    pub fn new(input: String) -> Result<Self, MyError> {
        let mut lexer: Lexer = Lexer::new(&input);
        let tokens: Vec<Token> = lexer.tokenize();
        let parser: Parser = Parser::new(tokens);
        let mut pos = Pos::new();
        parser.parse_whereclause(&mut pos, Token::EOF)
    }

    pub fn relate(&self, cols: &Vec<String>, vals: &Vec<String>) -> bool {
        match self {
            Self::True => true,
            Self::Relation(comp) => comp.relate(cols, vals),
            Self::And(expr1, expr2) => expr1.relate(cols, vals) && expr2.relate(cols, vals),
            Self::Or(expr1, expr2) => expr1.relate(cols, vals) || expr2.relate(cols, vals),
        }
    }
}

use crate::cql::lexer::*;
use crate::cql::parser::*;
use crate::cql::relation::Relation;
use std::error::Error;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum WhereClause {
    True,
    Relation(Relation),
    And(Box<WhereClause>, Box<WhereClause>),
    Or(Box<WhereClause>, Box<WhereClause>),
}

impl WhereClause {
    pub fn new(input: String) -> Result<Self, Box<dyn Error>> {
        let mut lexer: Lexer = Lexer::new(&input);
        let tokens: Vec<Token> = lexer.tokenize()?;
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

    pub fn find_value_for_key(&self, key: &str) -> Option<String> {
        match self {
            WhereClause::True => None, // Nothing to find in True
            WhereClause::Relation(Relation::Equal(left, right)) if left == key => {
                Some(right.clone()) // Found the desired relation, return the value
            }
            WhereClause::Relation(_) => None, // Other relations don't match
            WhereClause::And(lhs, rhs) | WhereClause::Or(lhs, rhs) => {
                // Recursively search both sides
                lhs.find_value_for_key(key).or_else(|| rhs.find_value_for_key(key))
            }
        }
    }
}

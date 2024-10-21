use crate::cql::lexer::*;
use crate::cql::relation::Relation;
use crate::cql::where_clause::WhereClause;
use crate::my_errors::MyError;

#[derive(Debug, PartialEq)]
pub struct Pos {
    pos: usize,
}

impl Default for Pos {
    fn default() -> Self {
        Self::new()
    }
}

impl Pos {
    pub fn new() -> Self {
        Pos { pos: 0 }
    }
    pub fn _move(&mut self) {
        self.pos += 1;
    }
    pub fn head(&self) -> usize {
        self.pos
    }
    pub fn previous(&self) -> usize {
        self.pos - 1
    }
    pub fn next(&self) -> usize {
        self.pos + 1
    }
}

#[derive(Debug, PartialEq)]
pub struct Parser {
    tokens: Vec<Token>,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens }
    }

    pub fn show(&self, pos: usize) -> &Token {
        &self.tokens[pos]
    }

    pub fn parse_relation(&self, pos: &mut Pos) -> Result<Relation, MyError> {
        let arg1;
        if let Token::Identifier(name) = self.show(pos.head()) {
            arg1 = name.to_string();
        } else {
            return Err(MyError::InvalidSyntax(
                "Invalid Syntax: <comparison> 1".to_string(),
            ));
        }
        pos._move();
        if !matches!(
            *self.show(pos.head()),
            Token::EqualTo
                | Token::NotEqualTo
                | Token::GreaterThan
                | Token::GreaterOrEqualThan
                | Token::LessThan
                | Token::LessOrEqualThan
        ) {
            return Err(MyError::InvalidSyntax(
                "Invalid Syntax: <Relation> 2".to_string(),
            ));
        }
        pos._move();
        let arg2;
        if let Token::Identifier(name) = self.show(pos.head()) {
            arg2 = name.to_string();
        } else {
            return Err(MyError::InvalidSyntax(
                "Invalid Syntax: <Relation> 3".to_string(),
            ));
        }
        let relation;
        match *self.show(pos.previous()) {
            Token::EqualTo => relation = Relation::Equal(arg1, arg2),
            Token::NotEqualTo => relation = Relation::NotEqual(arg1, arg2),
            Token::GreaterThan => relation = Relation::GreaterThan(arg1, arg2),
            Token::GreaterOrEqualThan => relation = Relation::GreaterOrEqualThan(arg1, arg2),
            Token::LessThan => relation = Relation::LessThan(arg1, arg2),
            Token::LessOrEqualThan => relation = Relation::LessOrEqualThan(arg1, arg2),
            _ => {
                return Err(MyError::InvalidSyntax(
                    "Invalid Syntax: <Relation>".to_string(),
                ))
            }
        }
        pos._move();
        Ok(relation)
    }

    pub fn parse_whereclause(&self, pos: &mut Pos, _last: Token) -> Result<WhereClause, MyError> {
        match *self.show(pos.head()) {
            Token::Lpar => {
                pos._move();
                let expr = self.parse_whereclause(pos, Token::Rpar)?;
                if self.show(pos.previous()) != &Token::Rpar {
                    return Err(MyError::InvalidSyntax(
                        "Invalid Sysntax: <where_clause> with unbalanced parentheses".to_string(),
                    ));
                }
                pos._move();
                Ok(expr)
            }
            Token::Identifier(_) => {
                let rela = self.parse_relation(pos)?;
                match self.show(pos.head()) {
                    Token::And => {
                        pos._move();
                        let arg2 = self.parse_whereclause(pos, _last)?;
                        Ok(WhereClause::And(
                            Box::new(WhereClause::Relation(rela)),
                            Box::new(arg2),
                        ))
                    }
                    Token::Or => {
                        pos._move();
                        let arg2 = self.parse_whereclause(pos, _last)?;
                        Ok(WhereClause::Or(
                            Box::new(WhereClause::Relation(rela)),
                            Box::new(arg2),
                        ))
                    }
                    Token::EOF => Ok(WhereClause::Relation(rela)),
                    _last => Ok(WhereClause::Relation(rela)),
                }
            }
            _ => Err(MyError::InvalidSyntax(
                "Invalid Sysntax: <where_clause> 2".to_string(),
            )),
        }
    }
}

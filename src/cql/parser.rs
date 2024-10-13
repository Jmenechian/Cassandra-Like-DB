use std::error::Error;
use crate::my_errors::MyError;
use crate::where_clause::WhereClause;
use crate::relation::Relation;

#[derive(Debug, PartialEq)]
pub struct Pos {
    pos: usize,
}

impl Pos {
    pub fn new() -> Self {
        Pos{ pos: 0 }
    }
    pub fn _move(&mut self) -> () {
        self.pos += 1;
    }
    pub fn head(& self) -> usize {
        self.pos
    }
    pub fn previous(& self) -> usize {
        self.pos-1
    }
    pub fn next(& self) -> usize {
        self.pos+1
    }
}

#[derive(Debug, PartialEq)]
pub struct Parser<'a> {
    tokens: &'a Vec<Token<'a>>,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: &'a Vec<Token>) -> Self {
        Parser { tokens }
    }

    pub fn show(&self, pos: usize) -> &Token<'a> {
        &self.tokens[pos]
    }

    pub fn parse_relation(&'a self, pos: &mut Pos) -> Result<Relation, Box<dyn Error>> {
        let arg1;
        if let Token::Identifier(name) = *self.show(pos.head()) {
            arg1 = name;
        } else {
            return Err(Box::new(MyError::InvalidSyntax("Invalid Syntax: <comparison> 1")))
        }
        pos._move();
        if !matches! (
            *self.show(pos.head()),
            Token::EqualTo
            | Token::NotEqualTo
            | Token::GreaterThan
            | Token::GreaterOrEqualThan
            | Token::LessThan
            | Token::LessOrEqualThan
        ) {
            return Err(Box::new(MyError::InvalidSyntax("Invalid Syntax: <Relation> 2")))
        }
        pos._move();
        let arg2;
        if let Token::Identifier(name) = *self.show(pos.head()) {
            arg2 = name;
        } else {
            return Err(Box::new(MyError::InvalidSyntax("Invalid Syntax: <Relation> 3")))
        }
        let relation;
        match *self.show(pos.previous()) {
            Token::EqualTo => relation = Relation::Equal(arg1, arg2),
            Token::NotEqualTo => relation = Relation::NotEqual(arg1, arg2),
            Token::GreaterThan => relation = Relation::GreaterThan(arg1, arg2),
            Token::GreaterOrEqualThan => relation = Relation::GreaterOrEqualThan(arg1, arg2),
            Token::LessThan =>relation = Relation::LessThan(arg1, arg2),
            Token::LessOrEqualThan =>relation = Relation::LessOrEqualThan(arg1, arg2),
            _ => return Err(Box::new(MyError::InvalidSyntax("Invalid Syntax: <Relation>")))
        }
        pos._move();
        Ok(relation)
    }

    pub fn parse_whereclause(&'a self, pos: &mut Pos, end: Token<'a>) -> Result<WhereClause, Box<dyn Error>> {
        let mut pos = Pos::new();
        match *self.show(pos.head()) {
            Token::Lpar => {
                pos._move();
                let expr = self.parse_whereclause(pos, Token::Rpar)?;
                if self.show(pos.previous()) != &Token::Rpar {
                    return Err(Box::new(MyError::InvalidSyntax("Invalid Sysntax: <where_clause> with unbalanced parentheses")))
                }
                pos._move();
                return Ok(expr)
            }
            Token::Not => {
                pos._move();
                let whereclause = self.parse_whereclause(pos, end)?;
                return Ok(WhereClause::Not(Box::new(whereclause)))
            },
            Token::Identifier(_) => {
                let rela = self.parse_relation(pos)?;
                match self.show(pos.head()) {
                    Token::And => {
                        pos._move();
                        let arg2 = self.parse_whereclause(pos, end)?;
                        return Ok(WhereClause::And(Box::new(WhereClause::Relation(rela)), Box::new(arg2)))
                    },
                    Token::Or => {
                        pos._move();
                        let arg2 = self.parse_whereclause(pos, end)?;
                        return Ok(WhereClause::Or(Box::new(WhereClause::Relation(rela)), Box::new(arg2)))
                    },
                    Token::EOF => {
                        return Ok(WhereClause::Relation(rela))
                    }
                    end => {
                        return Ok(WhereClause::Relation(rela))
                    },
                    _ => return Err(Box::new(MyError::InvalidSyntax("Invalid Sysntax: <where_clause> 1")))
                }
            },
            _ => return Err(Box::new(MyError::InvalidSyntax("Invalid Sysntax: <where_clause> 2")))
        }
    }
}
use std::error::Error;
use crate::cql::my_errors::MyError;

#[derive(Debug, PartialEq)]
pub enum Token<'a> {

    Where,
    And,
    Or,
    Not,

    Identifier(&'a str),

    EqualTo,
    LessThan,
    GreaterThan,
    NotEqualTo,
    GreaterOrEqualThan,
    LessOrEqualThan,

    Lpar,
    Rpar,

    EOF,
}

#[derive(Debug)]
pub struct Lexer<'a> {
    input: &'a str,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Lexer { input }
    }

    fn next_word(&mut self) -> Option<&'a str> {
        self.input = self.input.trim_start();
        let delimiters = &[' ','"','\'', '=', '>', '<', '(', ')', ',', '!'];
        if self.input.starts_with('\'') {
            if let Some(end) = self.input[1..].find('\'') {
                let word = &self.input[1..=end];
                self.input = &self.input[(end+2)..];
                return Some(word);
            }
        }
        if self.input.starts_with('"') {
            if let Some(end) = self.input[1..].find('"') {
                let word = &self.input[1..=end];
                self.input = &self.input[(end+2)..];
                return Some(word);
            }
        }
        let limit = self.input.find(|c| delimiters.contains(&c));
        match limit {
            Some(pos) => {
                let word = &self.input[..pos];
                if pos == 0 {
                    let _first = self.input.chars().next()?;
                    let char = &self.input[..1];
                    self.input = &self.input[1..];
                    return Some(char);
                }
                self.input = &self.input[pos..];
                Some(word)
            }
            None => {
                let word = self.input;
                self.input = "";
                Some(word)
            }
        }
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, Box<dyn Error>> {
        let mut tokens: Vec<Token> = Vec::new();
        while let Some(word) = self.next_word() {
            match word {
                "" => {
                    tokens.push(Token::EOF);
                    return Ok(tokens)
                }
                " " => continue,

                "WHERE" => tokens.push(Token::Where),
                
                "AND" => tokens.push(Token::And),
                "OR" => tokens.push(Token::Or),
                "NOT" => tokens.push(Token::Not),
                "!" => {
                    if let Some(word) = self.next_word() {
                        match word {
                            "=" => tokens.push(Token::NotEqualTo),
                            _ => {
                                tokens.push(Token::NotEqualTo);
                                tokens.push(Token::Identifier(word))
                            }
                        }
                    }
                }
                "=" => tokens.push(Token::EqualTo),
                ">" => { 
                    if let Some(word) = self.next_word() {
                        match word {
                            "=" => tokens.push(Token::GreaterOrEqualThan),
                            _ => {
                                tokens.push(Token::GreaterThan);
                                tokens.push(Token::Identifier(word));
                            }
                        }
                    }
                },
                "<" => {
                    if let Some(word) = self.next_word() {
                        match word {
                            "=" => tokens.push(Token::LessOrEqualThan),
                            _ => {
                                tokens.push(Token::LessThan);
                                tokens.push(Token::Identifier(word))
                            }
                        }
                    }
                }

                "," => tokens.push(Token::Comma),
                "(" => tokens.push(Token::Lpar),
                ")" => tokens.push(Token::Rpar),

                _ => tokens.push(Token::Identifier(word)),
            }
        }
        Ok(tokens)
    }
}
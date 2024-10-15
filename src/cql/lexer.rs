use std::error::Error;

#[derive(Debug, PartialEq)]
pub enum Token {
    Where,
    And,
    Or,
    Not,

    Identifier(String),
    Comma,

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
pub struct Lexer {
    input: String,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        Lexer {
            input: input.to_string(),
        }
    }

    fn next_word(&mut self) -> Option<String> {
        self.input = self.input.trim_start().to_string();
        let delimiters = &[' ', '"', '\'', '=', '>', '<', '(', ')', ',', '!'];
        if self.input.starts_with('\'') {
            if let Some(end) = self.input[1..].find('\'') {
                let word = self.input[1..=end].to_string();
                self.input = self.input[(end + 2)..].to_string();
                return Some(word);
            }
        }
        if self.input.starts_with('"') {
            if let Some(end) = self.input[1..].find('"') {
                let word = self.input[1..=end].to_string();
                self.input = self.input[(end + 2)..].to_string();
                return Some(word);
            }
        }
        let limit = self.input.find(|c| delimiters.contains(&c));
        match limit {
            Some(pos) => {
                let word = self.input[..pos].to_string();
                if pos == 0 {
                    let _first = self.input.chars().next()?;
                    let char = self.input[..1].to_string();
                    self.input = self.input[1..].to_string();
                    return Some(char);
                }
                self.input = self.input[pos..].to_string();
                Some(word)
            }
            None => {
                let word = self.input.clone();
                self.input = "".to_string();
                Some(word)
            }
        }
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, Box<dyn Error>> {
        let mut tokens: Vec<Token> = Vec::new();
        while let Some(word) = self.next_word() {
            match word.as_str() {
                "" => {
                    tokens.push(Token::EOF);
                    return Ok(tokens);
                }
                " " => continue,

                "WHERE" => tokens.push(Token::Where),

                "AND" => tokens.push(Token::And),
                "OR" => tokens.push(Token::Or),
                "NOT" => tokens.push(Token::Not),
                "!" => {
                    if let Some(word) = self.next_word() {
                        match word.as_str() {
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
                        match word.as_str() {
                            "=" => tokens.push(Token::GreaterOrEqualThan),
                            _ => {
                                tokens.push(Token::GreaterThan);
                                tokens.push(Token::Identifier(word));
                            }
                        }
                    }
                }
                "<" => {
                    if let Some(word) = self.next_word() {
                        match word.as_str() {
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

#[derive(Debug, Eq, PartialEq)]
pub struct Delete {
    pub table: String,
    pub condition_tokens: String, // to create Delete without initializing Tok.
}

impl Delete {
    pub fn new() -> Delete {
        Delete { table: "".to_string(), condition_tokens: "".to_string() }
    }
}
use std::collections::HashMap;

#[derive(Debug, Eq, PartialEq)]
pub struct Update {
    pub table: String,
    pub set: HashMap<String, String>,
    pub condition_tokens: String,
}

impl Update {
    pub fn new() -> Update {
        Update { table: "".to_string(), set: HashMap::new(), condition_tokens: "".to_string() }
    }
}
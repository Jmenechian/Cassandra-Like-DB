use crate::my_errors::MyError;
use std::collections::HashMap;

#[derive(Debug, Eq, PartialEq)]
pub struct Update {
    pub table: String,
    pub set: HashMap<String, String>,
    pub num_node: String,
}

impl Default for Update {
    fn default() -> Self {
        Self::new()
    }
}

impl Update {
    pub fn new() -> Update {
        Update {
            table: "".to_string(),
            set: HashMap::new(),
            num_node: "0".to_string(),
        }
    }

    pub fn parse_update(&mut self, _query: &str, _address_node: &str) -> Result<(), MyError> {
        Ok(())
    }

    pub fn execute_command(&mut self) -> Result<String, MyError> {
        Ok("".to_string())
    }
}

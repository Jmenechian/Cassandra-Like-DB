use crate::my_errors::MyError;

#[derive(Debug, Eq, PartialEq)]
pub struct Delete {
    pub table: String,
    pub num_node: String,
}

impl Default for Delete {
    fn default() -> Self {
        Self::new()
    }
}

impl Delete {
    pub fn new() -> Delete {
        Delete {
            table: "".to_string(),
            num_node: "0".to_string(),
        }
    }

    pub fn parse_delete(&mut self, query: &str, address_node: &str) -> Result<(), MyError> {
        Ok(())
    }

    pub fn execute_command(&mut self) -> Result<String, MyError> {
        Ok("".to_string())
    }
}

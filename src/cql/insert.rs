#[derive(Debug, Eq, PartialEq)]
pub struct Insert {
    pub table: String,
    pub columns: Vec<String>,
    pub values: Vec<String>,
}

impl Insert {
    pub fn new() -> Insert {
        Insert { table: "".to_string(), columns: Vec::new(), values: Vec::new() }
    }
}
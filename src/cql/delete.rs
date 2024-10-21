// Representa las consultas del tipo DELETE
// Aca se parsea la consulta y se ejecuta
// To Do

#[derive(Debug, Eq, PartialEq)]
pub struct Delete {
    pub table: String,
    pub condition_tokens: String,
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
            condition_tokens: "".to_string(),
        }
    }
}

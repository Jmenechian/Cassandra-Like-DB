use crate::my_errors::MyError;

#[derive(Debug, PartialEq, Eq)]
pub struct CreateTable {
    pub table_name: String,
    pub primary_key: String,
    pub columns: Vec<String>,
    pub num_node: String,
}

impl Default for CreateTable {
    fn default() -> Self {
        Self::new()
    }
}

impl CreateTable {
    pub fn new() -> CreateTable {
        CreateTable {
            table_name: "".to_string(),
            primary_key: "".to_string(),
            columns: Vec::new(),
            num_node: "0".to_string(),
        }
    }

    pub fn parse_create_table(&mut self, query: &str) -> Result<(), MyError> {
        if !query.contains("CREATE TABLE IF NOT EXISTS") {
            return Err(MyError::InvalidSyntax("Create table query not valid".to_string()));
        }

        let mut parts: Vec<&str> = query.split_whitespace().collect();
        if parts < 6 || parts[0] != "CREATE" || parts[1] != "TABLE" || parts[2] != "IF" || parts[3] != "NOT" || parts[4] != "EXISTS" {
            return Err(MyError::InvalidSyntax("Create table query not valid".to_string()));
        }

        // por ahora no acepta nombre de tablas con espacios
        let table_name = parts[5].to_string().replace("'", "");

        // por ahora no acepta nombres de columnas con espacios
        self.columns = self.get_columns(&query);

        let primary_key = self.get_primary_key(&query);

        Ok(())
    }

    fn get_columns(&self, query: &str) -> Vec<String> {
        Vec::new()
    }

    fn get_primary_key(&self, query: &str) -> String {
        "".to_string()
    }

    pub fn execute_command(&mut self) -> Result<String, MyError> {
        Ok("".to_string())
    }
}
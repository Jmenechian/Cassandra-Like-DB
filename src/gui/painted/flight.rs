#[derive(Debug)]
pub struct Flight {
    name: String,
}

impl Flight {
    pub fn new(name: String) -> Self {
        Self {
            name
        }
    }
}

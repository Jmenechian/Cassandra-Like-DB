use super::airport::Airport;

#[derive(Debug)]
pub struct Flight {
    pub name: String,
    pub origin_airport: Airport,
    pub return_airport: Airport,
}

impl Flight {
    pub fn new(name: String, origin_airport: Airport, return_airport: Airport) -> Self {
        Self {
            name,
            origin_airport,
            return_airport,
        }
    }
}

use std::fmt;

#[derive(Debug)]
pub enum AirportErrors {
    AirportIsNotAtDB(),
}

impl fmt::Display for AirportErrors {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            AirportErrors::AirportIsNotAtDB() => {
                write!(f, "ERROR: The airport is not found in our database")
            }
        }
    }
}

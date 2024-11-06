#[derive(Debug, PartialEq)]
/// Estados de un vuelo.
pub enum FlightStatus {
    Scheduled,
    Delayed,
    Cancelled,
    Boarding,
    Landed,
}

impl FlightStatus {
    /// Se parsea un string a un estado de vuelo indicado.
    pub fn match_status(status: String) -> FlightStatus {
        match status.to_lowercase().as_str() {
            "scheduled" => FlightStatus::Scheduled,
            "delayed" => FlightStatus::Delayed,
            "cancelled" => FlightStatus::Cancelled,
            "boarding" => FlightStatus::Boarding,
            "landed" => FlightStatus::Landed,
            _ => FlightStatus::Cancelled,
        }
    }

    pub fn to_str(&self) -> &str {
        match self {
            FlightStatus::Scheduled => "scheduled",
            FlightStatus::Delayed => "delayed",
            FlightStatus::Cancelled => "cancelled",
            FlightStatus::Boarding => "boarding",
            FlightStatus::Landed => "landed",
        }
    }
}

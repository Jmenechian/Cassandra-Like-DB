#[derive(Debug, PartialEq)]
/// Estados de un vuelo.
pub enum FlightStatus {
    Scheduled,
    Delayed,
    Cancelled,
    Boarding, // En origen
    Airborne, // En viaje a destino
    Landed, // En destino
}

impl FlightStatus {
    /// Se parsea un string a un estado de vuelo indicado.
    pub fn match_status(status: String) -> FlightStatus {
        match status.to_lowercase().as_str() {
            "scheduled" => FlightStatus::Scheduled,
            "delayed" => FlightStatus::Delayed,
            "cancelled" => FlightStatus::Cancelled,
            "boarding" => FlightStatus::Boarding,
            "airborne" => FlightStatus::Airborne,
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
            FlightStatus::Airborne => "airborne",
            FlightStatus::Landed => "landed",
        }
    }
}

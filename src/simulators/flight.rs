use super::flight_status::FlightStatus;
use crate::gui::painted::{airport::Airport, airport_coordinates::obtain_middle_position};

#[derive(Debug)]
/// Estructura que representa un vuelo, con su nombre, el aeropuesto de origen y el de destino, y otras variables del recorrido que hace.
pub struct Flight {
    pub number: String,
    pub origin_airport: Airport,
    pub return_airport: Airport,
    pub status: FlightStatus,
    pub departure_date: String,
    pub speed: f32,
    pub latitude: f32,
    pub longitude: f32,
    pub height: f32,
    pub midpoint: (f32, f32), // punto medio entre origen y destino: (latitud, longitud)
}

impl Flight {
    /// Se crea un nuevo vuelo con los datos indicados.
    pub fn new(
        number: String,
        origin_airport: Airport,
        return_airport: Airport,
        status: String,
        departure_date: String,
    ) -> Self {
        let status = FlightStatus::match_status(status);
        let speed = 0.0;
        let latitude = origin_airport.coordinates.0;
        let longitude = origin_airport.coordinates.1;
        let height = 0.0;
        let midpoint = obtain_middle_position(
            origin_airport.coordinates.0,
            return_airport.coordinates.0,
            origin_airport.coordinates.1,
            return_airport.coordinates.0,
        );

        Self {
            number,
            origin_airport,
            return_airport,
            status,
            departure_date,
            speed,
            latitude,
            longitude,
            height,
            midpoint,
        }
    }

    pub fn set_status(&mut self, new_status: FlightStatus) {
        self.status = new_status;
    }
}

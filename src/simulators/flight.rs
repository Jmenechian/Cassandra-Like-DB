use super::{airport::Airport, airport_coordinates::obtain_middle_position, flight_status::FlightStatus};
use crate::config::{CRUISING_SPEED, FLIGHT_STEP, SPEED_ALMOST_0, STEPS_AT_CRUISING_SPEED};

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
    pub step: (f32, f32), // paso en el que avanzará el avion en latitud y longitud
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
        let step = (Flight::get_flight_step_x(latitude, midpoint.0), Flight::get_flight_step_y(longitude, midpoint.1));

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
            step,
        }
    }

    pub fn set_status(&mut self, new_status: FlightStatus) {
        self.status = new_status;
    }

    pub fn update_speed(&mut self) {
        if self.latitude < (self.return_airport.coordinates.0 - (STEPS_AT_CRUISING_SPEED * self.step.0)) 
            && self.longitude < (self.return_airport.coordinates.1 - (STEPS_AT_CRUISING_SPEED * self.step.1))  {
            // Si no se llego a la cercania del punto medio (rango medio): aumentamos la velocidad, 
            //en punto medio tiene que llegar a los 840 km/h (velocidad crucero).
            // Tomamos el rango medio como la mitad de la recta que une origen y destino +- 2 step de vuelo.
            
            self.speed += CRUISING_SPEED / (FLIGHT_STEP - STEPS_AT_CRUISING_SPEED);  
            // Tenemos FLIGHT_STEP - STEPS_AT_CRUISING_SPEED pasos para llegar a la velocidad crucero.

        } else if self.latitude > (self.return_airport.coordinates.0 + (STEPS_AT_CRUISING_SPEED * self.step.0)) 
        && self.longitude > (self.return_airport.coordinates.1 + (STEPS_AT_CRUISING_SPEED * self.step.1))  {
            // Si pasamos el rango medio donde viajamos a velocidad crucero: disminuimos la velocidad.
            self.speed -= CRUISING_SPEED / (FLIGHT_STEP - STEPS_AT_CRUISING_SPEED);  
            // Tenemos FLIGHT_STEP - STEPS_AT_CRUISING_SPEED pasos para llegar a velocidad 0km/h.
        } 

        if self.speed <= SPEED_ALMOST_0 {
            self.speed = 0.0;
        }
    }

    // TODO:
    pub fn update_position(&mut self) {
        // TODO: Ver dependiendo de en que cuadrante estamos y dónde está el (0,0) en la GUI!!
        self.latitude += self.step.0;
        self.longitude += self.step.1;

        // TODO: Si posicion esta cerca de destino, a un 5% entonces ubicarla en destino.

    }

    // TODO: similar a velocidad 
    pub fn update_height(&mut self) {

    }

    fn get_flight_step_x(x1: f32, x2: f32) -> f32 {
        let distance = (x2 - x1).abs(); // longitud de la recta que unuria el punto de origen del avion y el punto medio del recorrido.
        distance / FLIGHT_STEP
    }

    fn get_flight_step_y(y1: f32, y2: f32) -> f32 {
        let distance = (y2 - y1).abs(); // longitud de la recta que unuria el punto de origen del avion y el punto medio del recorrido.
        distance / FLIGHT_STEP
    }
}

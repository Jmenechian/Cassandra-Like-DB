use super::{
    airport::Airport,
    coordinates::{get_angle, obtain_middle_position},
    flight_status::FlightStatus,
};
use crate::{
    config::{CRUISING_HEIGHT, FLIGHT_STEP, STEPS_AT_CRUISING_SPEED},
    errors::simulator_errors::SimulatorErrors,
};

#[derive(Debug, Clone)]
/// Estructura que representa un vuelo, con su número, el aeropuesto de origen y el de destino, y otras variables que varian durante recorrido que hace.
pub struct Flight {
    pub number: String,
    pub origin_airport: Airport,
    pub return_airport: Airport,
    pub status: FlightStatus,
    pub departure_date: String,
    pub speed: f32,
    pub latitude: f32,  // Eje y
    pub longitude: f32, // Eje x
    pub height: f32,
    pub midpoint: (f32, f32), // punto medio entre origen y destino: (longitud, latitud)
    pub step: (f32, f32),     // paso en el que avanzará el avion en longitud y latitud
    pub avg_speed: f32,       // velocidad estimada promedio
}

impl Flight {
    /// Se crea un nuevo vuelo con los datos indicados.
    pub fn new(
        number: String,
        origin_airport: Airport,
        return_airport: Airport,
        status: String,
        departure_date: String,
        avg_speed: String,
    ) -> Result<Self, SimulatorErrors> {
        let status = FlightStatus::match_status(status);
        let speed = 0.0;
        let longitude = origin_airport.coordinates.0;
        let latitude = origin_airport.coordinates.1;
        let height = 0.0;
        let midpoint = obtain_middle_position(
            origin_airport.coordinates.0,
            return_airport.coordinates.0,
            origin_airport.coordinates.1,
            return_airport.coordinates.1,
        );
        let step = (
            Flight::get_flight_step(longitude, midpoint.0),
            Flight::get_flight_step(latitude, midpoint.1),
        );
        let avg_speed = match avg_speed.parse::<f32>() {
            Ok(speed) => speed,
            Err(_) => {
                return Err(SimulatorErrors::Error(
                    "No se increso un valor de velocidad primedio correcta".to_string(),
                ));
            }
        };

        Ok(Self {
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
            avg_speed,
        })
    }

    /// Asignamos el status indicado al vuelo.
    pub fn set_status(&mut self, new_status: FlightStatus) {
        self.status = new_status;
    }

    /// Si no se llego a la cercania del punto medio (rango medio): aumentamos la velocidad,
    /// en punto medio tiene que llegar a la velocidad promedio.
    /// Tomamos el rango medio como la mitad de la recta que une origen y destino +- 2 step de vuelo.
    /// Tenemos FLIGHT_STEP - STEPS_AT_CRUISING_SPEED pasos para llegar a la velocidad crucero.
    /// Si pasamos el rango medio donde viajamos a velocidad crucero: disminuimos la velocidad.
    /// Tenemos FLIGHT_STEP - STEPS_AT_CRUISING_SPEED pasos para llegar a velocidad 0km/h.
    pub fn update_speed(&mut self) {
        if self.flight_in_destination() {
            self.speed = 0.0;
            return;
        }

        let zone = self.flight_vs_midpoint();
        if zone == -1 && self.speed < self.avg_speed {
            self.speed += self.avg_speed / (FLIGHT_STEP - STEPS_AT_CRUISING_SPEED);
        } else if zone == 0 {
            self.speed = self.avg_speed;
        } else if zone == 1 && self.speed > 0.0 {
            self.speed -= self.avg_speed / (FLIGHT_STEP - STEPS_AT_CRUISING_SPEED);
        }

        if self.speed < 0.0 {
            self.speed = 0.0;
        }
    }

    /// Se actualiza la posición del vuelo para hacercarlo en un step al aeropuesto de destino.
    /// Dependiendo de en que cuadrante estamos y la dirección del vuelo tenemos que sumar o restar a cada coordenadas.
    /// Hemisferio SUR -> latitud negativa (y<0)            Hemisferio NORTE -> latitud positiva (y>0)
    /// Hemisferio OESTE -> longitud negativa (x<0)         Hemisferio ESTE -> longitud positiva (x>0)
    pub fn update_position(&mut self) {
        if self.flight_in_destination() {
            return;
        }

        let angle = get_angle(
            self.origin_airport.coordinates.0,
            self.return_airport.coordinates.0,
            self.origin_airport.coordinates.1,
            self.return_airport.coordinates.1,
        );

        // step.0: paso en que se avanza en el eje x - Longitud eje x
        // step.1: paso en el que se avanza en el eje y - Latitud eje y
        if angle == 0.0 {
            // Aumentamos x:
            self.longitude += self.step.0;
        } else if angle < 90.0 {
            // Aumentamos x e y:
            self.longitude += self.step.0;
            self.latitude += self.step.1;
        } else if angle == 90.0 {
            // Aumentamos y:
            self.latitude += self.step.1
        } else if (90.0..180.0).contains(&angle) {
            // Disminuimos x y aumentamos y:
            self.longitude -= self.step.0;
            self.latitude += self.step.1;
        } else if angle == 180.0 {
            // Disminuimos x:
            self.longitude -= self.step.0;
        } else if (180.0..270.0).contains(&angle) {
            // Disminuimos x e y:
            self.longitude -= self.step.0;
            self.latitude -= self.step.1;
        } else if angle == 270.0 {
            // Disminuimmos y:
            self.latitude -= self.step.1;
        } else if (270.0..360.0).contains(&angle) {
            // Aumentamos x y disminuimos y:
            self.longitude += self.step.0;
            self.latitude -= self.step.1;
        }

        if self.flight_in_destination() {
            self.longitude = self.return_airport.coordinates.0;
            self.latitude = self.return_airport.coordinates.1;
        }
    }

    /// Similar a velocidad.
    pub fn update_height(&mut self) {
        if self.flight_in_destination() {
            self.height = 0.0;
            return;
        }

        let zone = self.flight_vs_midpoint();
        if zone == -1 && self.height < CRUISING_HEIGHT {
            // Si no se llego a la cercania del punto medio (rango medio): aumentamos la altura,
            // en punto medio tiene que llegar a los 10mil metros (altura crucero).
            // Tomamos el rango medio como la mitad de la recta que une origen y destino +- 2 step de vuelo.

            self.height += CRUISING_HEIGHT / (FLIGHT_STEP - STEPS_AT_CRUISING_SPEED);
            // Tenemos FLIGHT_STEP - STEPS_AT_CRUISING_SPEED pasos para llegar a la altura crucero.
        } else if zone == 0 {
            self.height = CRUISING_HEIGHT;
        } else if zone == 1 && self.height > 0.0 {
            // Si pasamos el rango medio donde viajamos a altura crucero: disminuimos la altura.
            self.height -= CRUISING_HEIGHT / (FLIGHT_STEP - STEPS_AT_CRUISING_SPEED);
            // Tenemos FLIGHT_STEP - STEPS_AT_CRUISING_SPEED pasos para llegar a la altura de 0 metros.
        }

        if self.height < 0.0 {
            self.height = 0.0;
        }
    }

    /// Si posicion del vuelo esta cerca de destino, es decir, a menos del 5% del trayecto total, entonces nos ubicamos en destino.
    /// Decimos un 5% porque dividimos el trayecto en 20 secciones (FLIGHT_STEP * 2.0).
    pub fn flight_in_destination(&mut self) -> bool {
        if (self.longitude == self.return_airport.coordinates.0)
            && (self.latitude == self.return_airport.coordinates.1)
        {
            return true;
        }

        // Calcula la distancia entre dos puntos (x1, y1) y (x2, y2)
        // Entre el aeropuesto de origen y el de destino:
        let dist_origin_and_dest =
            ((self.return_airport.coordinates.0 - self.origin_airport.coordinates.0).powi(2)
                + (self.return_airport.coordinates.1 - self.origin_airport.coordinates.1).powi(2))
            .sqrt();
        // Calculo la distancia máxima para considerar que llegó a destino:
        let upper_bound = dist_origin_and_dest / (FLIGHT_STEP * 2.0); // distance_origin_and_dest * percentage //let percentage = 0.05;

        // Distancia entre la posición del vuelo y el aeropuesto de destino:
        let distance_f_and_dest = ((self.return_airport.coordinates.0 - self.longitude).powi(2)
            + (self.return_airport.coordinates.1 - self.latitude).powi(2))
        .sqrt();

        if distance_f_and_dest < upper_bound {
            self.longitude = self.return_airport.coordinates.0;
            self.latitude = self.return_airport.coordinates.1;
            return true;
        }
        false
    }

    /// Devuelvo -1 si estamos antes de la zona media (crucero).
    /// 0 si estamos en esta zona.
    /// 1 si pasamos la zona de velocidad y altura crucero.
    ///     |--------|-----|-----|--------|
    ///     o       -m     m     +m       d
    ///         -1   |     0     |    1     (return)
    /// Entre -m y *m: velocidad y altitud crucero.
    /// m: midpoint
    fn flight_vs_midpoint(&mut self) -> i8 {
        // -m
        let long_min_speed_c = self.midpoint.0 - (STEPS_AT_CRUISING_SPEED * self.step.0);
        let lat_min_speed_c = self.midpoint.1 - (STEPS_AT_CRUISING_SPEED * self.step.1);

        // +m
        let long_max_speed_c = self.midpoint.0 + (STEPS_AT_CRUISING_SPEED * self.step.0);
        let lat_max_speed_c = self.midpoint.1 + (STEPS_AT_CRUISING_SPEED * self.step.1);

        // Calcula la distancia entre dos puntos (x1, y1) y (x2, y2):
        // - Entre el aeropuesto de origen y -m:
        let dist_origin_and_m_min = ((self.origin_airport.coordinates.0 - long_min_speed_c)
            .powi(2)
            + (self.origin_airport.coordinates.1 - lat_min_speed_c).powi(2))
        .sqrt();
        // - Entre el aeropuesto de origen y +m:
        let dist_origin_and_m_max = ((self.origin_airport.coordinates.0 - long_max_speed_c)
            .powi(2)
            + (self.origin_airport.coordinates.1 - lat_max_speed_c).powi(2))
        .sqrt();

        // Distancia entre el aeropuesto de origen y la posición del vuelo:
        let dist_origin_flight = ((self.origin_airport.coordinates.0 - self.longitude).powi(2)
            + (self.origin_airport.coordinates.1 - self.latitude).powi(2))
        .sqrt();

        if dist_origin_flight < dist_origin_and_m_min {
            return -1;
        } else if dist_origin_flight > dist_origin_and_m_max {
            return 1;
        }
        0
    }

    /// Devolvemos el tamaño de los pasos que hara el avion para que en total haya FLIGHT_STEP cantidad de pasos entre el origen y el punto medio,
    /// y la misma cantidad entre el punto medio y el destino.
    /// Es decir, en total el avion hara 2*FLIGHT_STEP pasos.
    fn get_flight_step(z1: f32, z2: f32) -> f32 {
        let distance = (z2 - z1).abs(); // longitud/longitud de la recta que unuria el punto de origen del avion y el punto medio del recorrido.
        distance / FLIGHT_STEP
    }

    /// Se actuliza la posicion actual del vuelo
    pub fn set_coordinates(&mut self, longitude: f32, latitude: f32) {
        self.longitude = longitude;
        self.latitude = latitude;
    }
}

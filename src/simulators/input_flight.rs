use chrono::Utc;

use crate::errors::simulator_errors::SimulatorErrors;

use super::user_input::ask_user_flight_values;

#[derive(Debug)]
/// Representa un vuelo auxiliar donde se guardan los valores que el usuario ingresó por consola.
pub struct InputFlight {
    pub flight_number: String,
    pub origin: String,
    pub destination: String,
    pub avg_speed: String,
    pub date: String,
}

impl InputFlight {
    pub fn new() -> Result<Self, SimulatorErrors> {
        if let Ok(values) = ask_user_flight_values() {
            let flight_number = values[0].clone();
            let origin = values[1].clone();
            let destination = values[2].clone();
            let avg_speed = values[3].clone();

            let today_utc = Utc::now();
            let date = today_utc.format("%y-%m-%d").to_string();

            return Ok(Self {
                flight_number,
                origin,
                destination,
                avg_speed,
                date,
            });
        }

        Err(SimulatorErrors::InputError())
    }
}

use super::flight::Flight;
use crate::{
    config::TABLE_PATH,
    errors::airport_errors::AirportErrors,
    gui::painted::{airport::Airport, airport_coordinates::get_airport_coordinates},
    my_errors::MyError,
};

use rand::{seq::SliceRandom, Rng};
use std::{
    error::Error,
    fs::File,
    io::{BufRead, BufReader},
};

#[derive(Debug, Clone)]
pub struct FlightSimulator {
    flights: Vec<Flight>,
}

impl Default for FlightSimulator {
    fn default() -> Self {
        Self::new()
    }
}

impl FlightSimulator {
    pub fn new() -> Self {
        Self { flights: vec![] }
    }

    pub fn run(&mut self) -> Result<(), MyError> {
        // Hacemos CREATE de la tabla flights.csv como CLient

        while let Ok(flight) = self.get_random_flight() {
            // Escribimos el vuelo en la tabla flights.csv --> hacemos INSERT como Client

            // actualizamos el estado del vuelo a 'Boarding', y en sus corresponientes tablas de origen y destino

            self.flights.push(flight);

            // Iteramos por los flights:

            // Se hacen los siguientes UPDATES:

            // Si no se llego al punto medio: aumentamos la velocidad , en punto medio tiene que llegar a  los 840 km/h (velocidad crucero)

            // Si pasamos el punto medio: disminuimos la velocidad

            // Si el avion llego a destino:

            // velocidad = 0

            // buscamos este vuelo en las tablas de estados origen y destino, y actualizamos su estado a 'Landed'

            // Aumentamos su posicion

            // esperamos 2 seguntos y continueamos con el while
        }

        Ok(())
    }

    /// Obtenemos un vuelo de una tabla random de origen_<n> que no este en estado "Cancelled".
    /// Lo inicializamos con velocidad 0 y en la posicion del aeropuesto de origen.
    fn get_random_flight(&self) -> Result<Flight, AirportErrors> {
        let mut found_plane = false;
        let mut line_flight = vec![];
        while !found_plane {
            let table_name = "origin";
            let partition = rand::thread_rng().gen_range(1..=8);
            let table_path = format!("{}/{}_{}.csv", TABLE_PATH, table_name, partition);

            if let Ok(line) = self.get_random_line(table_path) {
                line_flight = line;
                found_plane = true;
            }
        }

        if let Ok((airport_origin, airport_return)) = self.get_airports(&line_flight) {
            let flight = Flight::new(
                line_flight[3].clone(),
                airport_origin,
                airport_return,
                line_flight[4].clone(),
                line_flight[2].clone(),
            );
            return Ok(flight);
        }

        Err(AirportErrors::AirportIsNotAtDB())
    }

    /// Se retorna los valores de un vuelo de la tabla origin: [airport,destination,date,flight,status,timestamp].
    /// Este no tiene estado "Cancelled".
    fn get_random_line(&self, file_path: String) -> Result<Vec<String>, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);

        let mut lines: Vec<String> = Vec::new();
        for linea in reader.lines() {
            let linea = linea?;
            lines.push(linea);
        }

        if lines.is_empty() {
            return Err("File is empty.".into());
        }

        let mut values: Vec<String> = vec![];
        while values.is_empty() || values[4] == "Cancelled" {
            let mut rng = rand::thread_rng();
            let line = lines
                .choose(&mut rng)
                .ok_or("Error when selecting a random line")?;
            values = line.split(",").map(|l| l.to_string()).collect();
        }

        Ok(values.clone())
    }

    fn get_airports(&self, values: &[String]) -> Result<(Airport, Airport), AirportErrors> {
        let airports = get_airport_coordinates();

        if let (Some(airport_o), Some(airport_r)) =
            (airports.get(&values[0]), airports.get(&values[1]))
        {
            return Ok((airport_o.clone(), airport_r.clone()));
        }

        Err(AirportErrors::AirportIsNotAtDB())
    }
}

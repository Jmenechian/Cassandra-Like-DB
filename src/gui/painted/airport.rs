use std::{collections::HashMap, net::TcpStream};

use crate::{clientstuff::client_cql::Client, my_errors::MyError};
use super::flight::Flight;

#[derive(Debug, Clone, PartialEq)]
pub struct Airport {
    pub name: String,
    pub coordinates: (f32, f32), // (latitud, longitud)
}

impl Airport {
    pub fn default() -> Self {
        let name = "".to_string();
        let coordinates = (0.0, 0.0);
        Self {
            name,
            coordinates,
        }
    }

    pub fn new(name: String, coordinates: (f32, f32)) -> Self {
        Self {
            name,
            coordinates,
        }
    }

    pub fn get_origin_flights_from(&self, stream: &mut Option<TcpStream>, date: &str, airports: &HashMap<String, Airport>) -> Result<Vec<Flight>, MyError> {
        let query = format!("SELECT * FROM origin WHERE date = {} AND airport = {}", date, self.name);
        println!("query: {:?}", query);
        if let Some(stream) = stream {
            Client::send_message(stream, query).map_err(|_| MyError::SendingMessageError("Error sending query".to_string()))?;

            let response = Client::read_response(stream).map_err(|_| MyError::SendingMessageError("Error sending query".to_string()))?;
            println!("---- response{:?}", response);

            let mut flights: Vec<Flight> = vec![];
            for line in response {
                if line.is_empty() {
                    continue;
                }
                let values: Vec<String> = line.split(",").map(|f| f.to_string()).collect();

                let mut coordinates = (0.0, 0.0);
                if let Some(airport_aux) = airports.get(&values[0]) {
                    coordinates = airport_aux.coordinates.clone();
                }
                let airport_o = Airport::new(values[0].clone(), coordinates);

                let mut coordinates = (0.0, 0.0);
                if let Some(airport_aux) = airports.get(&values[1]) {
                    coordinates = airport_aux.coordinates.clone();
                }
                let airport_r = Airport::new(values[1].clone(), coordinates);

                flights.push(Flight::new(values[3].clone(), airport_o, airport_r));
            }
            println!("origin_flights: {:?}", flights);
            return Ok(flights);
        }

        Err(MyError::SendingMessageError("Error sending query".to_string()))
    }

    pub fn get_flights_that_return_to(&self, stream: &mut Option<TcpStream>, date: &str, airports: &HashMap<String, Airport>) -> Result<Vec<Flight>, MyError>{
        let query = format!("SELECT * FROM destination WHERE date = '{}' AND airport = '{}'", date, self.name);

        if let Some(stream) = stream {
            Client::send_message(stream, query).map_err(|_| MyError::SendingMessageError("Error sending query".to_string()))?;

            let response = Client::read_response(stream).map_err(|_| MyError::SendingMessageError("Error sending query".to_string()))?;
            println!("---- response{:?}", response);
            
            let mut flights: Vec<Flight> = vec![];
            for line in response {
                if line.is_empty() {
                    continue;
                }
                let values: Vec<String> = line.split(",").map(|f| f.to_string()).collect();

                let mut coordinates = (0.0, 0.0);
                if let Some(airport_aux) = airports.get(&values[1]) {
                    coordinates = airport_aux.coordinates.clone();
                }
                let airport_o = Airport::new(values[1].clone(), coordinates);

                let mut coordinates = (0.0, 0.0);
                if let Some(airport_aux) = airports.get(&values[0]) {
                    coordinates = airport_aux.coordinates.clone();
                }
                let airport_r = Airport::new(values[0].clone(), coordinates);
                
                flights.push(Flight::new(values[3].clone(), airport_o, airport_r));

            }
            println!("flights_that_return: {:?}", flights);
            return Ok(flights);
        }

        Err(MyError::SendingMessageError("Error sending query".to_string()))
    }
}

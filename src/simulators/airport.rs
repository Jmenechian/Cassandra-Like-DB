use std::{collections::HashMap, net::TcpStream};

use super::flight::Flight;
use crate::{clientstuff::client_cql::Client, my_errors::MyError};

#[derive(Debug, Clone, PartialEq)]
/// Estructura que representa un aeropuesto, con su nombre y coordenadas "geográficas" (en el mapa de la interfaz).
pub struct Airport {
    pub name: String,
    pub coordinates: (f32, f32), // (logitud, latitud) = (x,y)
}

impl Airport {
    /// Se crea un aeropuesto vacío.
    pub fn default_new() -> Self {
        let name = "".to_string();
        let coordinates = (0.0, 0.0);
        Self { name, coordinates }
    }

    /// Se crea un aeropuesto con los datos indicados.
    pub fn new(name: String, coordinates: (f32, f32)) -> Self {
        Self { name, coordinates }
    }

    pub fn do_query(&self, stream: &mut Option<TcpStream>, query: String) -> Result<(), MyError> {
        if let Some(stream) = stream {
            Client::send_message(stream, query)
                .map_err(|_| MyError::SendingMessageError("Error sending query".to_string()))?;

            let response = Client::read_response(stream)
                .map_err(|_| MyError::SendingMessageError("Error sending query".to_string()))?;
            println!("response: {:?}", response);
        }
        Ok(())
    }
    /// Se obtienen los vuelos cuyo aeropuerto de origen es el indicado.
    /// Se especifica la fecha en la que salen esos vuelos.
    pub fn get_origin_flights_from(
        &self,
        stream: &mut Option<TcpStream>,
        date: &str,
        airports: &HashMap<String, Airport>,
    ) -> Result<Vec<Flight>, MyError> {
        let query = format!(
            "SELECT * FROM origin WHERE date = {} AND airport = {}",
            date, self.name
        );
        println!("query: {:?}", query);
        if let Some(stream) = stream {
            Client::send_message(stream, query)
                .map_err(|_| MyError::SendingMessageError("Error sending query".to_string()))?;

            let response = Client::read_response(stream)
                .map_err(|_| MyError::SendingMessageError("Error sending query".to_string()))?;

            let mut flights: Vec<Flight> = vec![];
            for line in response {
                if line.is_empty() || line == "linea fue agregada" {
                    //ojo no se si rompe algo esto
                    continue;
                }

                // [origin, destination, date, flight, status, timestamp]
                let values: Vec<String> = line.split(",").map(|f| f.to_string()).collect();

                if values.len() < 2 {
                    continue;
                }

                let mut coordinates = (0.0, 0.0);
                if let Some(airport_aux) = airports.get(&values[0]) {
                    coordinates = airport_aux.coordinates;
                }
                let airport_o = Airport::new(values[0].clone(), coordinates);

                let mut coordinates = (0.0, 0.0);
                if let Some(airport_aux) = airports.get(&values[1]) {
                    coordinates = airport_aux.coordinates;
                }
                let airport_r = Airport::new(values[1].clone(), coordinates);

                let flight = match Flight::new(
                    values[3].clone(),
                    airport_o,
                    airport_r,
                    values[4].clone(),
                    values[2].clone(),
                    0.0.to_string(),
                ) {
                    Ok(f) => f,
                    Err(e) => {
                        return Err(MyError::Error(e.to_string()));
                    }
                };
                flights.push(flight);
            }
            println!("origin_flights: {:?}", flights);
            return Ok(flights);
        }

        Err(MyError::SendingMessageError(
            "Error sending query".to_string(),
        ))
    }

    /// Se obtienen los vuelos cuyo aeropuerto de retorno es el indicado.
    /// Se especifica la fecha en la que llegan esos vuelos.
    pub fn get_flights_that_return_to(
        &self,
        stream: &mut Option<TcpStream>,
        date: &str,
        airports: &HashMap<String, Airport>,
    ) -> Result<Vec<Flight>, MyError> {
        let query = format!(
            "SELECT * FROM destination WHERE date = {} AND airport = {}",
            date, self.name
        );

        if let Some(stream) = stream {
            Client::send_message(stream, query)
                .map_err(|_| MyError::SendingMessageError("Error sending query".to_string()))?;

            let response = Client::read_response(stream)
                .map_err(|_| MyError::SendingMessageError("Error sending query".to_string()))?;

            let mut flights: Vec<Flight> = vec![];
            for line in response {
                if line.is_empty() || line == "linea fue agregada" {
                    //ojo no se si rompe algo esto
                    continue;
                }
                let values: Vec<String> = line.split(",").map(|f| f.to_string()).collect();

                if values.len() < 2 {
                    continue;
                }

                let mut coordinates = (0.0, 0.0);
                if let Some(airport_aux) = airports.get(&values[1]) {
                    coordinates = airport_aux.coordinates;
                }
                let airport_o = Airport::new(values[1].clone(), coordinates);

                let mut coordinates = (0.0, 0.0);
                if let Some(airport_aux) = airports.get(&values[0]) {
                    coordinates = airport_aux.coordinates;
                }
                let airport_r = Airport::new(values[0].clone(), coordinates);

                let flight = match Flight::new(
                    values[3].clone(),
                    airport_o,
                    airport_r,
                    values[4].clone(),
                    values[2].clone(),
                    0.0.to_string(),
                ) {
                    Ok(f) => f,
                    Err(e) => {
                        return Err(MyError::Error(e.to_string()));
                    }
                };
                flights.push(flight);
            }
            println!("flights_that_return: {:?}", flights);
            return Ok(flights);
        }

        Err(MyError::SendingMessageError(
            "Error sending query".to_string(),
        ))
    }
}

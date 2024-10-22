use std::net::TcpStream;

use crate::{clientstuff::client_cql::Client, my_errors::MyError};

use super::flight::{self, Flight};

#[derive(Debug)]
#[derive(Clone)]
pub struct Airport {
    pub name: String,
    pub coordinates: (f32, f32), // (latitud, longitud)
}

impl Airport {
    pub fn new(name: String, coordinates: (f32, f32)) -> Self {
        Self {
            name,
            coordinates,
        }
    }

    pub fn get_origin_flights_from(&self, stream: &Option<TcpStream>, date: &str) -> Result<Vec<Flight>, MyError> {
        let query = format!("SELECT * FROM origin WHERE date = {} and airport = {}", date, self.name);
        println!("query: {:?}", query);
        if let Some(stream) = &stream {
            println!("Por enviar mensaje...");
            Client::send_message(stream, query).map_err(|_| MyError::SendingMessageError("Error sending query".to_string()))?;

            let response = Client::read_response(stream).map_err(|_| MyError::SendingMessageError("Error sending query".to_string()))?;
            let flights = response.message.split(",").map(|f| Flight::new(f.to_owned())).collect();
            println!("get_origin_flights_from: {:?}", flights);
            return Ok(flights);
        }

        Err(MyError::SendingMessageError("Error sending query".to_string()))
    }

    pub fn get_flights_that_return_to(&self, stream: &Option<TcpStream>, date: &str) -> Result<Vec<Flight>, MyError>{
        let query = format!("SELECT * FROM destination WHERE date = {} and airport = {}", date, self.name);

        if let Some(stream) = &stream {
            Client::send_message(stream, query).map_err(|_| MyError::SendingMessageError("Error sending query".to_string()))?;

            let response = Client::read_response(stream).map_err(|_| MyError::SendingMessageError("Error sending query".to_string()))?;
            let flights = response.message.split(",").map(|f| Flight::new(f.to_owned())).collect();
            println!("get_flights_that_return_to: {:?}", flights);
            return Ok(flights);
        }

        Err(MyError::SendingMessageError("Error sending query".to_string()))
    }
}

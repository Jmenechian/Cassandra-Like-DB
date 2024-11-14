use super::{
    airport::Airport, airport_coordinates::get_airport_coordinates, flight::Flight,
    user_input::ask_user_flight_values,
};
use crate::{
    client_cql::Client,
    config::N_WORKERS,
    errors::{airport_errors::AirportErrors, simulator_errors::SimulatorErrors},
    my_errors::MyError,
    simulators::flight_status::FlightStatus,
};

use chrono::Utc;
use std::{net::TcpStream, thread, time::Duration};
use threadpool::ThreadPool;

#[derive(Debug, Clone)]
/// Estructura que creará y simulará un vuelo desde su programación, pasando por su despegue hasta su llegada a destino.
pub struct FlightSimulator {
    flight_number: String,
    origin: String,
    destination: String,
    avg_speed: String,
    date: String,
}

impl FlightSimulator {
    fn new(flight_number: String, origin: String, destination: String, avg_speed: String) -> Self {
        let today_utc = Utc::now();
        let date = today_utc.format("%y-%m-%d").to_string();

        Self {
            flight_number,
            origin,
            destination,
            avg_speed,
            date,
        }
    }
    /// Pone en marcha la creación de vuelos y su simulación de despegue y llegada a destino.
    /// Se utilizan thread pool.
    pub fn run() -> Result<(), SimulatorErrors> {
        if FlightSimulator::create_flight_table().is_err() {
            println!("Las tablas ya estaban creadas.\n");
        } else {
            println!("Se creo la tabla flights_in_progress correctamente.\n");
        }

        let pool = ThreadPool::new(N_WORKERS);

        loop {
            // Pasamos el stream a cada thread para que pueda hacer las queries correspondientes.
            let mut stream_server: TcpStream = if let Ok(stream) = Client::connect_with_node() {
                stream
            } else {
                println!("No nos pudimos conectarnos a un nodo.\n");
                return Err(SimulatorErrors::AllNodesDisconnected());
            };

            let [mut _flight_number, mut _origin, mut _destination, mut _avg_speed] = [
                "".to_string(),
                "".to_string(),
                "".to_string(),
                "".to_string(),
            ];

            if let Ok(values) = ask_user_flight_values() {
                _flight_number = values[0].clone();
                _origin = values[1].clone();
                _destination = values[2].clone();
                _avg_speed = values[3].clone();
            } else {
                return Err(SimulatorErrors::InputError());
            }

            pool.execute(move || {
                let flight_simulator =
                    FlightSimulator::new(_flight_number, _origin, _destination, _avg_speed);

                match flight_simulator.obtain_flight_and_simulate_it(&mut stream_server) {
                    Ok(_) => {}
                    Err(e) => {
                        print!("{}", e);
                    }
                }
            });

            thread::sleep(Duration::from_secs(5));
        }
    }

    /// Se crea la tabla flights_in_progress.
    fn create_flight_table() -> Result<(), MyError> {
        // Buscamos un nodo conectado:
        let mut stream_server: TcpStream = if let Ok(stream) = Client::connect_with_node() {
            stream
        } else {
            return Err(MyError::TcpStreamError(
                "No nos pudimos conectarnos a un nodo.\n".to_string(),
            ));
        };

        let query = "CREATE TABLE flights_in_progress (flight STR, airport STR, destination STR, status STR, date DATETIME, 
            speed STR, latitude STR, longitude STR, height STR, timestamp TIMESTAMP, PRIMARY KEY (number, origin, date) 
            WITH CLUSTERING ORDER BY (timestamp ASC)".to_string();
        Client::send_message(&mut stream_server, query)
            .map_err(|_| MyError::SendingMessageError("Error sending query".to_string()))?;

        Ok(())
    }

    fn obtain_flight_and_simulate_it(
        &self,
        mut stream_server: &mut TcpStream,
    ) -> Result<(), MyError> {
        match self.create_flight(&mut stream_server) {
            Ok(flight) => self.simulate_flight(flight, &mut stream_server),
            Err(e) => Err(MyError::Error(e.to_string())),
        }
    }

    /// Si el vuelo no está en las tablas de estados (origin, destnation) se crea en estado "scheduled".
    /// Lo inicializamos con velocidad 0 y en la posicion del aeropuesto de origen.
    fn create_flight(&self, mut stream_server: &mut TcpStream) -> Result<Flight, SimulatorErrors> {
        match self.flight_in_table(&mut stream_server, "flights_in_progress") {
            Ok(exists) => {
                if exists {
                    return Err(SimulatorErrors::FlightIsAlreadySimulated());
                }
            }
            Err(e) => {
                return Err(SimulatorErrors::Error(e.to_string()));
            }
        }

        if let Ok((airport_origin, airport_return)) = self.get_airports() {
            let flight = Flight::new(
                self.flight_number.clone(),
                airport_origin,
                airport_return,
                "scheduled".to_string(),
                self.date.clone(),
                self.avg_speed.clone(),
            )?;

            self.flight_in_status_table(stream_server, &flight, "origin")
                .map_err(|error| SimulatorErrors::Error(error.to_string()))?;
            self.flight_in_status_table(stream_server, &flight, "destination")
                .map_err(|error| SimulatorErrors::Error(error.to_string()))?;

            return Ok(flight);
        }

        Err(SimulatorErrors::UncreatedFlight())
    }

    fn flight_in_table(&self, mut stream: &mut TcpStream, table: &str) -> Result<bool, MyError> {
        let date = self.date.clone();
        let mut airport = self.origin.clone();
        if table == "destination" {
            airport = self.destination.clone();
        }
        let flight = self.flight_number.clone();

        let query = format!(
            "SELECT * FROM {} WHERE date = {} AND airport = {} AND flight = {}",
            table, date, airport, flight
        );

        Client::send_message(&mut stream, query)
            .map_err(|_| MyError::SendingMessageError("Error sending query".to_string()))?;

        let response = Client::read_response(&mut stream)
            .map_err(|_| MyError::SendingMessageError("Error sending query".to_string()))?;

        for line in response {
            if line.is_empty() {
                continue;
            } else {
                return Ok(true);
            }
        }

        Ok(false)
    }

    /// Verifica si el vuelo está en la tabla y en caso de no estar lo crea.
    fn flight_in_status_table(
        &self,
        mut stream: &mut TcpStream,
        flight: &Flight,
        table: &str,
    ) -> Result<(), MyError> {
        if let Ok(exists) = self.flight_in_table(&mut stream, table) {
            if !exists {
                let query = format!("INSERT INTO {} (airport, destination, date, flight, status) VALUES ({}, {}, {}, {}, {})", 
                table, flight.origin_airport.name, flight.return_airport.name, flight.departure_date, flight.number, flight.status.to_str());

                Client::send_message(&mut stream, query)
                    .map_err(|_| MyError::SendingMessageError("Error sending query".to_string()))?;
            }
        }
        Ok(())
    }

    fn get_airports(&self) -> Result<(Airport, Airport), AirportErrors> {
        let airports = get_airport_coordinates();

        if let (Some(airport_o), Some(airport_r)) =
            (airports.get(&self.origin), airports.get(&self.destination))
        {
            return Ok((airport_o.clone(), airport_r.clone()));
        }

        Err(AirportErrors::AirportIsNotAtDB())
    }

    /// Cada thread se acupará de crear un avión y actualizar su estado hasta llegar a destino.
    fn simulate_flight(
        &self,
        mut flight: Flight,
        mut stream: &mut TcpStream,
    ) -> Result<(), MyError> {
        // Escribimos el vuelo en la tabla flights_in_progress.csv -> hacemos INSERT como Client
        FlightSimulator::insert_flight(&flight, &mut stream)?;
        println!("\nSe inserto el vuelo {:?} correctamente.\n", flight);

        while flight.status != FlightStatus::Landed {
            thread::sleep(Duration::from_secs(2));
            // Se hacen los siguientes UPDATES:

            // Si vuelo en scheduled -> 3 UPDATE: actualizamos el estado del vuelo a 'Boarding', y en sus corresponientes tablas de origen y destino.
            if flight.status == FlightStatus::Scheduled {
                flight.set_status(FlightStatus::Boarding);
                FlightSimulator::update_flight(&flight, stream)?;
                println!("\nEl vuelo {:?} está por salir.\n", flight);
                continue;
            }

            // Si status es boarding -> actualizamos a Airborne
            if flight.status == FlightStatus::Boarding {
                flight.set_status(FlightStatus::Airborne);
                println!("\nEl vuelo {:?} despego.\n", flight);
            }

            flight.update_speed();

            flight.update_height();

            flight.update_position();

            println!("\n Vuelo despues de los update: {:?}", flight);

            // Si el avion llego a destino, acutializamos su estado por el correspondiente:
            if flight.flight_in_destination() {
                flight.set_status(FlightStatus::Landed);
                flight.update_speed();
                flight.update_height();
                FlightSimulator::update_flight(&flight, stream)?;
                println!("\nEl vuelo {:?} llegó a destino.\n", flight);
                break;
            }

            FlightSimulator::update_flight(&flight, stream)?;
            println!(
                "Se actualizo el vuelo {} en todas las tablas.",
                flight.status.to_str()
            );
        }
        Ok(())
    }

    fn insert_flight(flight: &Flight, mut stream: &mut TcpStream) -> Result<(), MyError> {
        let query = format!("INSERT INTO flights_in_progress (flight, airport, destination, status, date, speed, latitude, longitude, height) 
        VALUES ({}, {}, {}, {}, {}, {}, {}, {}, {})", flight.number, flight.origin_airport.name, flight.return_airport.name, flight.status.to_str(), flight.departure_date, flight.speed, flight.latitude, flight.longitude, flight.height);

        Client::send_message(&mut stream, query)
            .map_err(|_| MyError::SendingMessageError("Error sending query".to_string()))?;

        Ok(())
    }

    fn update_flight(flight: &Flight, mut stream: &mut TcpStream) -> Result<(), MyError> {
        for table in ["flights_in_progress", "origin", "destination"] {
            let airport = if table == "destination" {
                flight.return_airport.name.clone()
            } else {
                flight.origin_airport.name.clone()
            };

            let mut extra = "".to_string();
            if table == "flights_in_progress" {
                extra = format!(
                    ", speed = {}, latitude = {}, longitude = {}, height = {} ",
                    flight.speed, flight.latitude, flight.longitude, flight.height
                );
            }

            let query = format!(
                "UPDATE {} SET status = {}{} WHERE flight = {} AND airport = {} AND date = {}",
                table,
                flight.status.to_str(),
                extra,
                flight.number,
                airport,
                flight.departure_date
            );

            Client::send_message(&mut stream, query)
                .map_err(|_| MyError::SendingMessageError("Error sending query".to_string()))?;
        }

        Ok(())
    }
}

use super::{
    airport::Airport,
    coordinates::get_airport_coordinates,
    flight::Flight,
    queries::{
        get_query_create_simulator_table, get_query_insert_in_destination,
        get_query_insert_in_origin, get_query_insert_in_simulator, get_query_select_in_origin,
        get_query_update_in_destination, get_query_update_in_origin, get_query_update_in_simulator,
    },
    user_input::ask_user_flight_values,
};
use crate::{
    client_cql::Client,
    config::{self, N_WORKERS},
    errors::{
        airport_errors::AirportErrors, my_errors::MyError, simulator_errors::SimulatorErrors,
    },
    simulators::{flight_status::FlightStatus, queries::get_query_select_in_destination},
};

use chrono::Utc;
use std::{net::{IpAddr, TcpStream}, thread, time::Duration};
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

    fn initializer() -> Result<Self, SimulatorErrors> {
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

        let flight_simulator =
            FlightSimulator::new(_flight_number, _origin, _destination, _avg_speed);

        Ok(flight_simulator)
    }

    /// Pone en marcha la creación de vuelos y su simulación de despegue y llegada a destino.
    /// Se utilizan thread pool.
    pub fn run(ip: IpAddr, port: u16) -> Result<(), SimulatorErrors> {
        if FlightSimulator::create_flight_table(ip, port).is_err() {
            println!("Las tablas ya estaban creadas.\n");
        } else {
            println!("Se creo la tabla flights_in_progress correctamente.\n");
        }

        let pool = ThreadPool::new(N_WORKERS);

        loop {
            let mut stream_server: TcpStream = if let Ok(stream) = Client::connect_with_node(ip, port) {
                stream
            } else {
                return Err(SimulatorErrors::AllNodesDisconnected());
            };

            let flight_simulator = FlightSimulator::initializer()?;

            pool.execute(move || {
                match flight_simulator.obtain_flight_and_simulate_it(&mut stream_server) {
                    Ok(_) => {}
                    Err(e) => {
                        println!("{}", e);
                    }
                }
            });

            thread::sleep(Duration::from_secs(5));
        }
    }

    /// Se crea la tabla flights_in_progress.
    fn create_flight_table(ip: IpAddr, port: u16) -> Result<(), MyError> {
        // Buscamos un nodo conectado:
        let mut stream_server: TcpStream = if let Ok(stream) = Client::connect_with_node(ip, port) {
            stream
        } else {
            return Err(MyError::TcpStreamError(
                "No nos pudimos conectarnos a un nodo.\n".to_string(),
            ));
        };

        let query = get_query_create_simulator_table();
        Client::send_message(&mut stream_server, query, config::ONE)
            .map_err(|_| MyError::SendingMessageError("Error sending query".to_string()))?;

        Ok(())
    }

    fn obtain_flight_and_simulate_it(&self, stream_server: &mut TcpStream) -> Result<(), MyError> {
        match self.create_flight(stream_server) {
            Ok(flight) => self.simulate_flight(flight, stream_server),
            Err(e) => Err(MyError::Error(e.to_string())),
        }
    }

    /// Si el vuelo no está en las tablas de estados (origin, destnation) se crea en estado "scheduled".
    /// Lo inicializamos con velocidad 0 y en la posicion del aeropuesto de origen.
    fn create_flight(&self, stream_server: &mut TcpStream) -> Result<Flight, SimulatorErrors> {
        match self.flight_in_table(stream_server, "flights_in_progress") {
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

    fn flight_in_table(&self, stream: &mut TcpStream, table: &str) -> Result<bool, MyError> {
        let query = if table == "destination" {
            get_query_select_in_destination(
                self.date.clone(),
                self.destination.clone(),
                self.flight_number.clone(),
            )
        } else {
            get_query_select_in_origin(
                self.date.clone(),
                self.origin.clone(),
                self.flight_number.clone(),
            )
        };

        Client::send_message(stream, query, config::ONE)
            .map_err(|_| MyError::SendingMessageError("Error sending query".to_string()))?;

        let response = Client::read_response(stream)
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
        stream: &mut TcpStream,
        flight: &Flight,
        table: &str,
    ) -> Result<(), MyError> {
        if let Ok(exists) = self.flight_in_table(stream, table) {
            if !exists {
                let query = if table == "destination" {
                    get_query_insert_in_destination(flight)
                } else {
                    get_query_insert_in_origin(flight)
                };

                Client::send_message(stream, query, config::ONE)
                    .map_err(|_| MyError::SendingMessageError("Error sending query".to_string()))?;

                let _ = Client::read_response(stream)
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
    fn simulate_flight(&self, mut flight: Flight, stream: &mut TcpStream) -> Result<(), MyError> {
        // Escribimos el vuelo en la tabla flights_in_progress.csv -> hacemos INSERT como Client
        FlightSimulator::insert_flight(&flight, stream)?;
        println!("\nSe agrego el vuelo {:?} correctamente.\n", flight);

        while flight.status != FlightStatus::Landed {
            thread::sleep(Duration::from_secs(2));
            // Se hacen los siguientes UPDATES:

            // Si vuelo en scheduled -> 3 UPDATE: actualizamos el estado del vuelo a 'Boarding', y en sus corresponientes tablas de origen y destino.
            if flight.status == FlightStatus::Scheduled {
                flight.set_status(FlightStatus::Boarding);
                FlightSimulator::update_flight(&flight, stream)?;
                continue;
            }

            // Si status es boarding -> actualizamos a Airborne
            if flight.status == FlightStatus::Boarding {
                flight.set_status(FlightStatus::Airborne);
            }

            flight.update_speed();

            flight.update_height();

            flight.update_position();

            // Si el avion llego a destino, acutializamos su estado por el correspondiente:
            if flight.flight_in_destination() {
                flight.set_status(FlightStatus::Landed);
                flight.update_speed();
                flight.update_height();
                FlightSimulator::update_flight(&flight, stream)?;
            } else {
                FlightSimulator::update_flight(&flight, stream)?;
            }
        }
        Ok(())
    }

    fn insert_flight(flight: &Flight, stream: &mut TcpStream) -> Result<(), MyError> {
        let query = get_query_insert_in_simulator(flight);

        Client::send_message(stream, query, config::ONE)
            .map_err(|_| MyError::SendingMessageError("Error sending query".to_string()))?;

        let _ = Client::read_response(stream)
            .map_err(|_| MyError::SendingMessageError("Error sending query".to_string()))?;

        Ok(())
    }

    fn update_flight(flight: &Flight, stream: &mut TcpStream) -> Result<(), MyError> {
        for table in ["origin", "destination", "flights_in_progress"] {
            let query = match table {
                "origin" => get_query_update_in_origin(flight),
                "destination" => get_query_update_in_destination(flight),
                "flights_in_progress" => get_query_update_in_simulator(flight),
                _ => continue,
            };

            Client::send_message(stream, query, config::ONE)
                .map_err(|_| MyError::SendingMessageError("Error sending query".to_string()))?;

            let _ = Client::read_response(stream)
                .map_err(|_| MyError::SendingMessageError("Error sending query".to_string()))?;
        }

        Ok(())
    }
}

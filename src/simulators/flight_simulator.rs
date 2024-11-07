use super::{airport::Airport, airport_coordinates::get_airport_coordinates, flight::Flight};
use crate::{
    client_cql::Client,
    config::{N_WORKERS, TABLE_PATH},
    errors::airport_errors::AirportErrors,
    my_errors::MyError,
    simulators::flight_status::FlightStatus,
};

use rand::{seq::SliceRandom, Rng};
use std::{
    error::Error,
    fs::File,
    io::{BufRead, BufReader},
    net::TcpStream,
    sync::{Arc, RwLock}, thread, time::Duration,
};
use threadpool::ThreadPool;

#[derive(Debug, Clone)]
/// Estructura que creará y simulará un vuelo desde su programación, pasando por su despegue hasta su llegada a destino.
pub struct FlightSimulator {}


impl FlightSimulator {
    /// Pone en marcha la creación de vuelos y su simulación de despegue y llegada a destino.
    /// Se utilizan thread pool.
    pub fn run() {
        if let Err(e) = FlightSimulator::create_flight_table() {
            println!("Error al crear la tabla: {}", e);
            return;
        };
        println!("Se creo la tabla flights_in_progress correctamente.");

        let pool = ThreadPool::new(N_WORKERS);

        loop {
            // Pasamos el stream a cada thread para que pueda hacer las queries correspondientes.
            let stream_server: TcpStream = if let Ok(stream) = Client::connect_with_node() {
                stream
            } else {
                println!("No nos pudimos conectarnos a un nodo.");
                return;
            };
            let stream_server: Arc<RwLock<TcpStream>> = Arc::new(RwLock::new(stream_server));

            pool.execute(move || 
                match FlightSimulator::obtain_flight_and_simulate_it(stream_server) {
                    Ok(_) => {},
                    Err(e) => {
                        print!("{}", e);
                    }
                }
            );

            thread::sleep(Duration::from_secs(5));
        }
        // Borrar tabla flights_in_progress.
    }

    /// Se crea la tabla flights_in_progress.
    fn create_flight_table() -> Result<(), MyError> {
        // Buscamos un nodo conectado:
        let mut stream_server: TcpStream = if let Ok(stream) = Client::connect_with_node() {
            stream
        } else {
            return Err(MyError::TcpStreamError(
                "No nos pudimos conectarnos a un nodo.".to_string(),
            ));
        };

        let query = "CREATE TABLE flights_in_progress (flight STR, airport STR, destination STR, status STR, date DATETIME, 
            speed STR, latitude STR, longitude STR, height STR, timestamp TIMESTAMP, PRIMARY KEY (number, origin, date) 
            WITH CLUSTERING ORDER BY (timestamp ASC)".to_string();
        Client::send_message(&mut stream_server, query)
            .map_err(|_| MyError::SendingMessageError("Error sending query".to_string()))?;

        Ok(())
    }

    fn obtain_flight_and_simulate_it(stream_server: Arc<RwLock<TcpStream>>) -> Result<(), MyError> {

        match FlightSimulator::get_random_flight(&stream_server) {
            Ok(flight) => FlightSimulator::simulate_flight(flight, &stream_server),
            Err(e) => Err(MyError::Error(e.to_string()))
        }
    }

    /// Obtenemos un vuelo de una tabla random de origen_<n> que no este en estado "Cancelled".
    /// Lo inicializamos con velocidad 0 y en la posicion del aeropuesto de origen.
    fn get_random_flight(stream_server: &Arc<RwLock<TcpStream>>) -> Result<Flight, MyError> {
        let mut found_plane = false;
        let mut line_flight = vec![];
        while !found_plane {
            let table_name = "origin";
            let partition = rand::thread_rng().gen_range(1..=8);
            let table_path = format!("{}/{}_{}.csv", TABLE_PATH, table_name, partition);

            if let Ok(line) = FlightSimulator::get_random_line(table_path) {
                line_flight = line;
                found_plane = true;
            }

            if found_plane {
                match FlightSimulator::flight_is_already_simulated(&line_flight, stream_server) {
                    Ok(exists) => {
                        if exists {
                            found_plane = false;
                        }
                    },
                    Err(e) => {
                        return Err(e);
                    }
                }
            }
        }

        if let Ok((airport_origin, airport_return)) = FlightSimulator::get_airports(&line_flight) {
            let flight = Flight::new(
                line_flight[3].clone(),
                airport_origin,
                airport_return,
                line_flight[4].clone(),
                line_flight[2].clone(),
            );
            return Ok(flight);
        }

        Err(MyError::Error("No se pudo crear el vuelo de simulación".to_string()))
    }

    /// Se retorna los valores de un vuelo de la tabla origin: [airport,destination,date,flight,status,timestamp].
    /// Este no tiene estado "Cancelled".
    fn get_random_line(file_path: String) -> Result<Vec<String>, Box<dyn Error>> {
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
        while values.is_empty() || values[4].to_lowercase() != "scheduled" {
            let mut rng = rand::thread_rng();
            let line = lines
                .choose(&mut rng)
                .ok_or("Error when selecting a random line")?;
            values = line.split(",").map(|l| l.to_string()).collect();
        }

        Ok(values.clone())
    }

    fn flight_is_already_simulated(line_flight: &Vec<String>, stream: &Arc<RwLock<TcpStream>>) -> Result<bool, MyError> {

        let date = line_flight[4].clone(); 
        let airport = line_flight[1].clone();
        let flight = line_flight[0].clone(); 

        let query = format!(
            "SELECT * FROM flights_in_progress WHERE date = {} AND airport = {} AND flight = {}",
            date, airport, flight
        );

        let mut stream = stream
            .write()
            .map_err(|_| MyError::LockingError("Error adquiriendo el write lock".to_string()))?;

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

    fn get_airports(values: &[String]) -> Result<(Airport, Airport), AirportErrors> {
        let airports = get_airport_coordinates();

        if let (Some(airport_o), Some(airport_r)) =
            (airports.get(&values[0]), airports.get(&values[1]))
        {
            return Ok((airport_o.clone(), airport_r.clone()));
        }

        Err(AirportErrors::AirportIsNotAtDB())
    }

    /// Cada thread se acupará de crear un avión y actualizar su estado hasta llegar a destino.
    fn simulate_flight(
        mut flight: Flight,
        mut stream: &Arc<RwLock<TcpStream>>,
    ) -> Result<(), MyError> {

        // Escribimos el vuelo en la tabla flights_in_progress.csv -> hacemos INSERT como Client
        FlightSimulator::insert_flight(&flight, stream)?;
        println!("Se insertaron los valores correctamente.");

        while flight.status != FlightStatus::Landed {
            // Se hacen los siguientes UPDATES:

            // Si vuelo en scheduled -> 3 UPDATE: actualizamos el estado del vuelo a 'Boarding', y en sus corresponientes tablas de origen y destino.
            if flight.status == FlightStatus::Scheduled {
                flight.set_status(FlightStatus::Boarding);
                FlightSimulator::update_flight(&flight, stream)?;
                println!("El vuelo {:?} está por salir.",  flight);
                continue;
            }

            // Si status es boarding -> actualizamos a Airborne
            if flight.status == FlightStatus::Boarding {
                flight.set_status(FlightStatus::Airborne);
                println!("El vuelo {:?} despego.",  flight);
            }

            flight.update_position();

            flight.update_speed();

            flight.update_height();

            // Si el avion llego a destino:
            if flight.latitude == flight.return_airport.coordinates.0 && flight.longitude == flight.return_airport.coordinates.1 {
                flight.set_status(FlightStatus::Landed);
                FlightSimulator::update_flight(&flight, &mut stream)?;
                println!("El vuelo {:?} llegó a destino.", flight);
                break;
            }

            FlightSimulator::update_flight(&flight, &mut stream)?;
            println!("Se actualizo el vuelo en todas las tablas.");

            thread::sleep(Duration::from_secs(2));
        }
        Ok(())
    }

    fn insert_flight(
        flight: &Flight,
        stream: &Arc<RwLock<TcpStream>>,
    ) -> Result<(), MyError> {

        let mut stream = stream
            .write()
            .map_err(|_| MyError::LockingError("Error adquiriendo el write lock".to_string()))?;

        let query = format!("INSERT INTO flights_in_progress (flight, airport, destination, status, date, speed, latitude, longitude, height) 
        VALUES ({}, {}, {}, {}, {}, {}, {}, {}, {})", flight.number, flight.origin_airport.name, 
        flight.return_airport.name, flight.status.to_str(), flight.departure_date, flight.speed, flight.latitude, 
        flight.longitude, flight.height);

        Client::send_message(&mut stream, query)
            .map_err(|_| MyError::SendingMessageError("Error sending query".to_string()))?;
        
        Ok(())
    }

    fn update_flight(flight: &Flight, stream: &Arc<RwLock<TcpStream>>) -> Result<(), MyError> {
        let mut stream = stream
            .write()
            .map_err(|_| MyError::LockingError("Error adquiriendo el write lock".to_string()))?;

        for table in ["flights_in_progress", "origin", "destination"] {
            let airport;
            if table == "destination" {
                airport = flight.return_airport.name.clone();
            } else {
                airport = flight.origin_airport.name.clone();
            }

            let query = format!("UPDATE {} SET status = {}, speed = {}, latitude = {}, longitude = {}, height = {} WHERE flight = {} AND airport = {} AND date = {}", 
                table, flight.status.to_str(), flight.speed, flight.latitude, flight.longitude, flight.height, flight.number, airport, flight.departure_date);

            Client::send_message(&mut stream, query)
                .map_err(|_| MyError::SendingMessageError("Error sending query".to_string()))?;
        }
        
        Ok(())
    }
}

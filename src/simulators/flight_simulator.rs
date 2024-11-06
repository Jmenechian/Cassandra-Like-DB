use super::flight::Flight;
use crate::{
    client_cql::Client,
    config::{N_WORKERS, RING_SIZE, TABLE_PATH},
    errors::airport_errors::AirportErrors,
    gui::painted::{airport::Airport, airport_coordinates::get_airport_coordinates},
    my_errors::MyError,
    simulators::flight_status::FlightStatus,
};

use rand::{seq::SliceRandom, Rng};
use std::{
    error::Error,
    fs::File,
    io::{BufRead, BufReader},
    net::TcpStream,
    sync::{Arc, RwLock},
};
use threadpool::ThreadPool;

#[derive(Debug, Clone)]
/// Estructura que creará y simulará un vuelo desde su programación, pasando por su despegue hasta su llegada a destino.
pub struct FlightSimulator {}

impl FlightSimulator {
    /// Pone en marcha la creación de vuelos y su simulación de despegue y llegada a destino.
    /// Se utilizan thread pool.
    pub fn run() {
        let _ = if let Err(e) = FlightSimulator::create_flight_table() {
            println!("Error al crear la tabla: {}", e);
            return;
        };

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

            pool.execute(move || match FlightSimulator::get_random_flight() {
                Ok(flight) => match FlightSimulator::update_flights(flight, stream_server) {
                    Ok(_) => {}
                    Err(e) => {
                        println!("Error al actualizar el estado del vuelo: {}", e);
                    }
                },
                Err(e) => {
                    println!("Error al obtener vuelo aleatorio: {}", e);
                }
            });
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

        let query = "CREATE TABLE flights_in_progress (number STR, airport STR, destination STR, status STR, date DATETIME, 
            speed STR, latitude STR, longitude STR, height STR, timestamp TIMESTAMP, PRIMARY KEY (number, origin, date) 
            WITH CLUSTERING ORDER BY (timestamp ASC)".to_string();
        Client::send_message(&mut stream_server, query)
            .map_err(|_| MyError::SendingMessageError("Error sending query".to_string()))?;
        println!("Se creo la tabla flights_in_progress correctamente.");
        Ok(())
    }

    /// Cada thread se acupará de crear un avión y actualizar su estado hasta llegar a destino.
    fn update_flights(
        flight: Flight,
        stream_server: Arc<RwLock<TcpStream>>,
    ) -> Result<(), MyError> {

        println!("flight: {:?}", flight);

        // Escribimos el vuelo en la tabla flights_in_progress.csv --> hacemos INSERT como Client
        FlightSimulator::insert_flight(&flight, stream_server)?;

        while flight.status != FlightStatus::Landed {
            println!("En while.");
            // Se hacen los siguientes UPDATES:

            // Si vuelo en scheduled -> 3 UPDATE: actualizamos el estado del vuelo a 'Boarding', y en sus corresponientes tablas de origen y destino.

            // Si no se llego al punto medio: aumentamos la velocidad , en punto medio tiene que llegar a  los 840 km/h (velocidad crucero)

            // Si pasamos el punto medio: disminuimos la velocidad

            // Si el avion llego a destino:

            // velocidad = 0

            // buscamos este vuelo en las tablas de estados origen y destino, y actualizamos su estado a 'Landed'

            // Aumentamos su posicion

            // esperamos 2 seguntos y continueamos con el while
            break; // borrar
        }
        Ok(())
    }

    fn insert_flight(
        flight: &Flight,
        stream_server: Arc<RwLock<TcpStream>>,
    ) -> Result<(), MyError> {
        let mut stream = stream_server
            .write()
            .map_err(|_| MyError::LockingError("Error adquiriendo el write lock".to_string()))?;

        let query = format!("INSERT INTO origin (number, airport, destination, status, date, speed, latitude, longitude, height) 
        VALUES ({}, {}, {}, {}, {}, {}, {}, {}, {})", flight.number, flight.origin_airport.name, 
        flight.return_airport.name, flight.status.to_str(), flight.departure_date, flight.speed, flight.latitude, 
        flight.longitude, flight.height);

        Client::send_message(&mut stream, query)
            .map_err(|_| MyError::SendingMessageError("Error sending query".to_string()))?;
        println!("Se insertaron los valores correctamente.");
        Ok(())
    }

    /// Obtenemos un vuelo de una tabla random de origen_<n> que no este en estado "Cancelled".
    /// Lo inicializamos con velocidad 0 y en la posicion del aeropuesto de origen.
    fn get_random_flight() -> Result<Flight, AirportErrors> {
        let mut found_plane = false;
        let mut line_flight = vec![];
        while !found_plane {
            let table_name = "origin";
            let partition = rand::thread_rng().gen_range(1..=8);
            let table_path = format!("{}/{}_{}.csv", TABLE_PATH, table_name, partition);
            println!("{}", table_path);

            if let Ok(line) = FlightSimulator::get_random_line(table_path) {
                line_flight = line;
                found_plane = true;
                println!("found_plane: {}", found_plane);
            }
        }
        println!("line_flight: {:?}", line_flight);
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

        Err(AirportErrors::AirportIsNotAtDB())
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
            println!("values: {:?}", values);
        }

        println!("values return: {:?}", values);
        Ok(values.clone())
    }

    fn get_airports(values: &[String]) -> Result<(Airport, Airport), AirportErrors> {
        let airports = get_airport_coordinates();

        if let (Some(airport_o), Some(airport_r)) =
            (airports.get(&values[0]), airports.get(&values[1]))
        {
            println!("airport_o: {:?} - airport_r: {:?}", airport_o, airport_r);
            return Ok((airport_o.clone(), airport_r.clone()));
        }

        Err(AirportErrors::AirportIsNotAtDB())
    }
}

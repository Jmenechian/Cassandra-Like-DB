use crate::{
    config::{ONE, QUORUM},
    errors::{
        airport_errors::AirportErrors, my_errors::MyError, simulator_errors::SimulatorErrors,
    },
    simulators::{
        airport::Airport,
        connection_server::ConnectionServer,
        coordinates::get_airport_coordinates,
        flight::Flight,
        flight_status::FlightStatus,
        flight_updater::FlightUpdater,
        input_flight::InputFlight,
        queries::{
            get_query_create_simulator_table, get_query_insert_in_destination,
            get_query_insert_in_origin, get_query_insert_in_simulator,
            get_query_select_in_destination, get_query_select_in_origin,
            get_query_select_in_simulator, get_query_update_in_destination,
            get_query_update_in_origin, get_query_update_in_simulator,
        },
    },
};

use std::sync::mpsc::Sender;
use std::{net::IpAddr, thread, time::Duration};

#[derive(Debug)]
/// Estructura que creará y simulará vuelos desde su programación, pasando por su despegue hasta su llegada a destino.
pub struct FlightSimulator {
    channel_updater_sender: Sender<(String, u16)>,
    connection_server: ConnectionServer,
}

impl FlightSimulator {
    fn new(channel_updater_sender: Sender<(String, u16)>, ip: IpAddr, port: u16) -> Result<Self, MyError> {
        let connection_server = ConnectionServer::initialize_with_any_node(ip, port)?;

        Ok(Self {
            channel_updater_sender,
            connection_server,
        })
    }

    /// Pone en marcha la creación de vuelos y su simulación desde despegue hasta la llegada a destino.
    pub fn run(ip: IpAddr, port: u16) -> Result<(), MyError> {
        if FlightSimulator::create_flight_table(ip, port).is_err() {

            println!("Las tablas ya estaban creadas.\n");
        } else {
            println!("Se creo la tabla flights_in_progress correctamente.\n");
        }

        // Se crea el modulo encargado de escuchar las queries y ejecutarlas:
        let mut flight_updater = FlightUpdater::new().map_err(|e| MyError::Error(e.to_string()))?;
        let channel_sender: Sender<(String, u16)> = flight_updater.get_channel_to_send();
        let path_queries = flight_updater.get_mutex_path();

        // Thread que escucha y guarda las queries:
        thread::spawn({
            move || match flight_updater.listen_queries() {
                Ok(_) => {}
                Err(e) => {
                    println!("{}", e);
                }
            }
        });

        // Thread que ejecuta las queries, usando Threadpools:
        thread::spawn({
            move || match FlightUpdater::update_flights(path_queries, ip, port) {
                Ok(_) => {}
                Err(e) => {
                    println!("{}", e);
                }
            }
        });

        // Loop encargado de simular los vuelos y enviar al modulo anterior las queries de update del estado y seguimiento del vuelo.
        // Se lanza un thread por cada vuelo que se simula:
        let mut i = 1;
        loop {
            println!("\n - Simulacion {}", i);
            let flight_simulator = FlightSimulator::new(channel_sender.clone(), ip, port)?;
            let input_flight = InputFlight::new().map_err(|e| MyError::Error(e.to_string()))?;
            thread::spawn({
                move || match flight_simulator.obtain_flight_and_simulate_it(&input_flight) {
                    Ok(_) => {}
                    Err(e) => {
                        println!("{}", e);
                    }
                }
            });
            i += 1;
        }
    }

    /// Se crea la tabla flights_in_progress.
    fn create_flight_table(ip: IpAddr, port: u16) -> Result<(), MyError> {
        let connection_server = ConnectionServer::initialize_with_any_node(ip, port)?;
        let channel = connection_server.get_channel_to_receive();

        let query = get_query_create_simulator_table();

        connection_server.make_query(query, QUORUM)?;
        channel
            .recv()
            .map_err(|e| MyError::ChannelReceiveError(e.to_string()))?;

        Ok(())
    }

    fn obtain_flight_and_simulate_it(&self, input_flight: &InputFlight) -> Result<(), MyError> {
        match self.create_flight(input_flight) {
            Ok(flight) => self.simulate_flight(flight),
            Err(e) => Err(MyError::Error(e.to_string())),
        }
    }

    /// Si el vuelo no está en las tablas de estados (origin, destnation) se crea en estado "scheduled".
    /// Lo inicializamos con velocidad 0 y en la posicion del aeropuesto de origen.
    fn create_flight(&self, input_flight: &InputFlight) -> Result<Flight, SimulatorErrors> {
        if let Ok((airport_origin, airport_return)) = self.get_airports(input_flight) {
            let flight = Flight::new(
                input_flight.flight_number.clone(),
                airport_origin,
                airport_return,
                "scheduled".to_string(),
                input_flight.date.clone(),
                input_flight.avg_speed.clone(),
            )?;

            match self.flight_in_table(&flight, "flights_in_progress") {
                Ok(exists) => {
                    if exists {
                        return Err(SimulatorErrors::FlightIsAlreadySimulated());
                    }
                }
                Err(e) => {
                    return Err(SimulatorErrors::Error(e.to_string()));
                }
            }

            self.flight_in_status_table(&flight, "origin")
                .map_err(|error| SimulatorErrors::Error(error.to_string()))?;
            self.flight_in_status_table(&flight, "destination")
                .map_err(|error| SimulatorErrors::Error(error.to_string()))?;

            return Ok(flight);
        }

        Err(SimulatorErrors::UncreatedFlight())
    }

    fn flight_in_table(&self, flight: &Flight, table: &str) -> Result<bool, MyError> {
        let query = if table == "destination" {
            get_query_select_in_destination(
                flight.departure_date.clone(),
                flight.return_airport.name.clone(),
                flight.number.clone(),
            )
        } else if table == "origin" {
            get_query_select_in_origin(
                flight.departure_date.clone(),
                flight.origin_airport.name.clone(),
                flight.number.clone(),
            )
        } else {
            get_query_select_in_simulator(
                flight.departure_date.clone(),
                flight.origin_airport.name.clone(),
                flight.number.clone(),
            )
        };

        let channel_receive = self.connection_server.get_channel_to_receive();
        let consistency = match table {
            "origin" | "destination" => QUORUM,
            "flights_in_progress" => ONE,
            _ => QUORUM,
        };
        self.connection_server.make_query(query, consistency)?;

        let response = channel_receive
            .recv()
            .map_err(|e| MyError::ChannelReceiveError(e.to_string()))?;

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
    fn flight_in_status_table(&self, flight: &Flight, table: &str) -> Result<(), MyError> {
        if let Ok(exists) = self.flight_in_table(flight, table) {
            if !exists {
                let query = if table == "destination" {
                    get_query_insert_in_destination(flight)
                } else {
                    get_query_insert_in_origin(flight)
                };

                let channel = self.connection_server.get_channel_to_receive();
                self.connection_server.make_query(query, QUORUM)?;
                channel
                    .recv()
                    .map_err(|e| MyError::ChannelReceiveError(e.to_string()))?;
            }
        }
        Ok(())
    }

    fn get_airports(
        &self,
        input_flight: &InputFlight,
    ) -> Result<(Airport, Airport), AirportErrors> {
        let airports = get_airport_coordinates();

        if let (Some(airport_o), Some(airport_r)) = (
            airports.get(&input_flight.origin),
            airports.get(&input_flight.destination),
        ) {
            return Ok((airport_o.clone(), airport_r.clone()));
        }

        Err(AirportErrors::AirportIsNotAtDB())
    }

    /// Cada thread se acupará de crear un avión y actualizar su estado hasta llegar a destino.
    fn simulate_flight(&self, mut flight: Flight) -> Result<(), MyError> {
        // Escribimos el vuelo en la tabla flights_in_progress.csv -> hacemos INSERT como Client
        self.insert_flight(&flight)?;
        println!("\nInicio de vuelo simulado:\n{:?}\n", flight);

        while flight.status != FlightStatus::Landed {
            thread::sleep(Duration::from_secs(2));
            // Se hacen los siguientes UPDATES:

            // Si vuelo en scheduled -> 3 UPDATE: actualizamos el estado del vuelo a 'Boarding', y en sus corresponientes tablas de origen y destino.
            if flight.status == FlightStatus::Scheduled {
                flight.set_status(FlightStatus::Boarding);
                self.update_flight(&flight)?;
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
                self.update_flight(&flight)?;
            } else {
                self.update_flight(&flight)?;
            }
        }
        Ok(())
    }

    fn insert_flight(&self, flight: &Flight) -> Result<(), MyError> {
        let query = get_query_insert_in_simulator(flight);

        let channel = self.connection_server.get_channel_to_receive();

        self.connection_server.make_query(query, ONE)?;

        channel
            .recv()
            .map_err(|e| MyError::ChannelReceiveError(e.to_string()))?;

        Ok(())
    }

    fn update_flight(&self, flight: &Flight) -> Result<(), MyError> {
        for table in ["origin", "destination", "flights_in_progress"] {
            let query = match table {
                "origin" => get_query_update_in_origin(flight),
                "destination" => get_query_update_in_destination(flight),
                "flights_in_progress" => get_query_update_in_simulator(flight),
                _ => continue,
            };

            let consistency = match table {
                "origin" | "destination" => QUORUM,
                "flights_in_progress" => ONE,
                _ => continue,
            };

            self.channel_updater_sender
                .send((query, consistency))
                .map_err(|e| MyError::ChannelSendError(e.to_string()))?;
        }

        Ok(())
    }
}

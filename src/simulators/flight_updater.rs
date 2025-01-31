use std::{
    net::{IpAddr, TcpStream},
    sync::{mpsc::Sender, Arc, Mutex},
};

use crate::{
    client_cql::Client,
    config::{N_WORKERS, QUERIES_PATH},
    errors::{my_errors::MyError, simulator_errors::SimulatorErrors},
    pool::threadpool::ThreadPool,
    utils::{append_row, get_and_remove_first_line, is_file_empty, open_file},
};

use super::channel_type::ChannelType;

#[derive(Debug)]
/// Modulo encargado en guardar las queries de UPDATE que se deben hacer sobre los vuelos que se estan simulando
/// en las tablas de estado y de seguimento de vuelos.
/// Además, ejecuta dichas queries utilizando Thread Pools de N_WORKERS.
/// El agregado de una nueva query estará Lockeado para no perder ninguna.
/// Los Threads estarán sincronizados por channel.
pub struct FlightUpdater {
    channel: ChannelType,
    queries_path: Arc<Mutex<String>>,
}

impl FlightUpdater {
    pub fn new() -> Result<Self, SimulatorErrors> {
        Ok(Self {
            channel: ChannelType::new(),
            queries_path: Arc::new(Mutex::new(QUERIES_PATH.to_string())),
        })
    }

    pub fn get_channel_to_send(&self) -> Sender<(String, u16)> {
        self.channel.sender.clone()
    }

    pub fn get_mutex_path(&self) -> Arc<Mutex<String>> {
        Arc::clone(&self.queries_path)
    }

    pub fn listen_queries(&mut self) -> Result<(), MyError> {
        loop {
            let (query, consistency) = self
                .channel
                .receiver
                .recv()
                .map_err(|e| MyError::ChannelReceiveError(e.to_string()))?;

            self.enqueue(query, consistency)?;
        }
    }

    /// Se agrega la query a la cola de ejecucion.
    fn enqueue(&mut self, query: String, consistency: u16) -> Result<(), MyError> {
        let path = self
            .queries_path
            .lock()
            .map_err(|e| MyError::LockingError(e.to_string()))?;

        let mut file = open_file(path.to_string())?;

        append_row(&mut file, &[query, consistency.to_string()])?;

        Ok(())
    }

    /// Devuelve la primer query encolada, y None en caso de no haber más queries.
    fn dequeue(path_queries: Arc<Mutex<String>>) -> Result<(String, u16), MyError> {
        let path = path_queries
            .lock()
            .map_err(|e| MyError::LockingError(e.to_string()))?;

        let result = get_and_remove_first_line(path.to_string())?;

        if result.len() >= 2 {
            let consistency = result[1]
                .parse::<u16>()
                .map_err(|e| MyError::Error(e.to_string()))?;
            return Ok((result[0].clone(), consistency));
        }

        Err(MyError::Error("Empty query file".to_string()))
    }

    /// Se ejecutan las queries.
    pub fn update_flights(path: Arc<Mutex<String>>, ip: IpAddr, port: u16) -> Result<(), MyError> {
        // Buscamos un nodo conectado:
        let mut client = Client::initialize(ip, port);
        let stream: TcpStream = if let Ok(stream) = client.connect() {
            stream
        } else {
            return Err(MyError::AllNodesDisconnected());
        };

        let threadpool = ThreadPool::build(N_WORKERS)?;

        loop {
            if is_file_empty(QUERIES_PATH) {
                continue;
            }

            let mut stream = stream
                .try_clone()
                .map_err(|e| MyError::TcpStreamError(e.to_string()))?;

            let path = Arc::clone(&path);

            threadpool.execute(move || {
                if let Ok((query, consistency)) = FlightUpdater::dequeue(path) {
                    let _ = FlightUpdater::make_query(&mut stream, query, consistency);
                }
            });
        }
    }

    /// Se ejecuta la query.
    /// Se envia por el channel la respuesta.
    fn make_query(stream: &mut TcpStream, query: String, consistency: u16) -> Result<(), MyError> {
        if Client::send_message(stream, query, consistency).is_ok() {
            Client::read_response(stream).map_err(|e| MyError::Error(e.to_string()))?;
        }

        Ok(())
    }
}

use std::{
    collections::VecDeque, 
    net::TcpStream, 
    sync::{mpsc::{self, Receiver, Sender}, Arc, Mutex}
};
use threadpool::ThreadPool;

use crate::{
    client_cql::Client, 
    config::N_WORKERS, 
    errors::{my_errors::MyError, simulator_errors::SimulatorErrors}
};

#[derive(Debug)]
/// Modulo encargado en guardar las queries de UPDATE que se deben hacer sobre los vuelos que se estan simulando 
/// en las tablas de estado y de seguimento de vuelos.
/// Además, ejecuta dichas queries utilizando Thread Pools de N_WORKERS.
/// El agregado de una nueva query estará Lockeado para no perder ninguna.
/// Los Threads estarán sincronizados por channel.
pub struct FlightUpdater {
    queries: Arc<Mutex<VecDeque<(String, u16)>>>,
    threadpool: ThreadPool,
    stream: TcpStream,
    channel: (Sender<(String, u16)>, Receiver<(String, u16)>),
}

impl FlightUpdater {

    pub fn new() -> Result<Self, SimulatorErrors> {
        let stream_server: TcpStream = if let Ok(stream) = Client::connect_with_node() {
            stream
        } else {
            return Err(SimulatorErrors::AllNodesDisconnected());
        };
        
        Ok(Self {
            queries: Arc::new(Mutex::new(VecDeque::new())),
            threadpool: ThreadPool::new(N_WORKERS),
            stream: stream_server,
            channel: mpsc::channel(), //(tx, rx) -> send, receive
        })
    }

    pub fn get_channel_to_send(&self) -> Sender<(String, u16)> {
        self.channel.0.clone()
    }

    pub fn listen_queries_and_execute(&mut self) -> Result<(), MyError> {
        loop {
            let (query, consistency) = self.channel.1.recv().map_err(|e| MyError::ChannelReceiveError(e.to_string()))?;
            self.enqueue(query, consistency)?;

            self.update_flights()?;
        }
    }

    /// Se agrega la query a la cola de ejecucion.
    fn enqueue(&mut self, query: String, consistency: u16) -> Result<(), MyError> {
        let mut queue = self.queries.lock().map_err(|e| MyError::LockingError(e.to_string()))?;
        queue.push_back((query, consistency));
        Ok(())
    }

    /// Devuelve la primer query encolada, y None en caso de no haber más queries.
    fn dequeue(&mut self) -> Result<Option<(String, u16)>, MyError> {
        let mut queue = self.queries.lock().map_err(|e| MyError::LockingError(e.to_string()))?;
        Ok(queue.pop_front())
    }

    /// Se ejecutan las queries.
    pub fn update_flights(&mut self) -> Result<(), MyError> {

        // TODO: el dequeue se tiene que hacer dentro del hilo! Por eso Mutex
        if let Some((query, consistency)) = self.dequeue()? {
            let mut stream = self.stream.try_clone().map_err(|e| MyError::TcpStreamError(e.to_string()))?;
            self.threadpool.execute(move || {
                let _ = FlightUpdater::make_query(&mut stream, query, consistency);
            });
        }

        Ok(())
    }

    /// Se ejecuta la query.
    /// Se envia por el channel la respuesta.
    fn make_query(mut stream: &mut TcpStream, query: String, consistency: u16) -> Result<(), MyError> {

        if Client::send_message(&mut stream, query, consistency).is_ok() {
            Client::read_response(&mut stream).map_err(|e| MyError::Error(e.to_string()))?;
        }

        println!("Ok send result query");
        Ok(())
    }
}
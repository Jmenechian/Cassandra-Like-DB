use std::{
    net::TcpStream,
    sync::mpsc::{self, Receiver, Sender},
    thread,
};

use crate::{client_cql::Client, errors::my_errors::MyError, simulators::airport::Airport};

/// Estructura que representa la conexión con el nodo.
/// Guarda el stream y los channels por los que va a enviar el resultado,
/// y por donde se va a poder recibir lo que envió.
#[derive(Debug)]
pub struct ConnectionServer {
    pub stream: Option<TcpStream>,
    pub channels: (Sender<Vec<String>>, Receiver<Vec<String>>),
    pub client: Client,
}

impl ConnectionServer {
    /// Se inicializa la conección con el nodo.
    pub fn initialize(mut client: Client) -> Self {
        let stream = if let Ok(stream) = client.connect() {
            Some(stream)
        } else {
            None
        };

        Self {
            stream,
            channels: mpsc::channel(), //(tx, rx) -> send, receive
            client,
        }
    }

    fn get_connection_server(&self) -> Result<TcpStream, MyError> {
        if let Some(stream) = &self.stream {
            if let Ok(stream) = stream.try_clone() {
                return Ok(stream);
            }
        }
        Err(MyError::TcpStreamError(
            "Error connecting with server".to_string(),
        ))
    }

    /// Devuelve el extreamo del channel por el que se va a poder recibir lo que envíe desde esta estructura.
    pub fn get_channel_to_receive(&self) -> &Receiver<Vec<String>> {
        &self.channels.1
    }

    /// Se lanza un thread para realizar las consultas de los vuelos que llegan y salen del aeropuesto.
    /// Luego se envía la respuesta por el channel.
    pub fn send_origin_and_return_flights(
        &self,
        date: String,
        airport: Airport,
    ) -> Result<(), MyError> {
        let stream_server: TcpStream = if let Ok(stream) = self.get_connection_server() {
            stream
        } else {
            return Err(MyError::TcpStreamError("Error".to_string()));
        };
        let stream_server_2: TcpStream = if let Ok(stream) = self.get_connection_server() {
            stream
        } else {
            return Err(MyError::TcpStreamError("Error".to_string()));
        };

        let tx = self.channels.0.clone();
        let logger = self.client.logger.clone();

        let _ = thread::spawn({
            move || {
                if let Ok(origin_flights) =
                    airport.get_origin_flights_from(&mut Some(stream_server), &date, logger.clone())
                {
                    let _ = tx.send(origin_flights);
                }
                if let Ok(return_flights) =
                    airport.get_flights_that_return_to(&mut Some(stream_server_2), &date, logger)
                {
                    let _ = tx.send(return_flights);
                }
            }
        });
        Ok(())
    }

    /// Se lanza un thread que realizará la query indicada al nodo con el que mantenemos la conexion.
    /// Se envia por el channel la respuesta.
    pub fn make_query(&self, query: String, consistency: u16) -> Result<(), MyError> {
        println!("\n- Query: {}\n", query);
        let mut stream_server: TcpStream = if let Ok(stream) = self.get_connection_server() {
            stream
        } else {
            return Err(MyError::TcpStreamError("Error in stream".to_string()));
        };

        let tx = self.channels.0.clone();
        let logger_send = self.client.logger.clone();
        let logger_read = self.client.logger.clone();

        let _ = thread::spawn({
            move || {
                if Client::send_message(&mut stream_server, query, consistency, logger_send).is_ok() {
                    if let Ok(response) = Client::read_response(&mut stream_server, logger_read) {
                        let _ = tx.send(response);
                    }
                }
            }
        });
        println!("Ok send result query");
        Ok(())
    }
}

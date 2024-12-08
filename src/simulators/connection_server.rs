use std::{
    net::TcpStream,
    sync::mpsc::{self, Receiver, Sender},
    thread,
};

use crate::{client_cql::Client, errors::my_errors::MyError};

/// Estructura que representa la conexión con el nodo.
/// Guarda el stream y los channels por los que va a enviar el resultado,
/// y por donde se va a poder recibir lo que envió.
#[derive(Debug)]
pub struct ConnectionServer {
    pub stream: Option<TcpStream>,
    pub channels: (Sender<Vec<String>>, Receiver<Vec<String>>),
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
        }
    }

    pub fn initialize_with_any_node() -> Result<Self, MyError> {
        // Buscamos un nodo conectado:
        let stream: Option<TcpStream> = if let Ok(stream) = Client::connect_with_node() {
            Some(stream)
        } else {
            return Err(MyError::TcpStreamError(
                    "No nos pudimos conectarnos a un nodo.\n".to_string(),
            ));
        };
    
        Ok(Self {
            stream,
            channels: mpsc::channel(), //(tx, rx) -> send, receive
        })
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

        let _ = thread::spawn({
            move || {
                if Client::send_message(&mut stream_server, query, consistency).is_ok() {
                    if let Ok(response) = Client::read_response(&mut stream_server) {
                        let _ = tx.send(response);
                    }
                }
            }
        });
        println!("Ok send result query");
        Ok(())
    }
}

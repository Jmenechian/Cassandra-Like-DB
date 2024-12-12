use crate::config::{self, RING_SIZE};
use crate::errors::my_errors::MyError;
use crate::messagestuff::message::Sendable;
use crate::messagestuff::message_cql::cqlenvelope::*;
use crate::messagestuff::message_cql::querymessage::QueryMessage;
use crate::messagestuff::message_cql::resultmessage::ResultBody;
use crate::loggerstuff::logger::Logger;
use std::io::{self, BufRead};
use std::net::TcpStream;
use std::sync::{Arc, RwLock};

use std::thread;

#[derive(Debug)]
/// Estructura que representará al cliente conectandose al server.
/// Podrá realizarse consultas al server y recibir su respuesta.
pub struct Client {
    pub server_ip: String,
    pub ip: String,
    pub logger: Arc<RwLock<Logger>>,
}

impl Client {
    /// Se inicializan los puertos del cliente.
    pub fn initialize(port: String) -> Result<Self, MyError> {

        let logger = match Logger::new("client".to_string()) { //////////////////// REVISAR!!!
            Ok(logger) => Arc::new(RwLock::new(logger)),
            Err(_) => return Err(MyError::LoggerError("Error al crear logger".to_string())),
        };

        Ok(Self {
            server_ip: format!("127.0.0.1:800{}", port),
            ip: "127.0.0.1:45450".to_string(),
            logger,
        })
    }

    /// Devuelve Ok si se puede conectar con el server.
    pub fn handle_input(&mut self) -> Result<(), MyError> {
        let mut _stream = TcpStream::connect(&self.server_ip).map_err(|e| MyError::TcpStreamError(e.to_string()))?;

        Ok(())
    }

    /// Se conecta con el server y devuelve el stream.
    pub fn connect(&mut self) -> Result<TcpStream, MyError> {
        if let Ok(stream) = TcpStream::connect(&self.server_ip) {
            Ok(stream)
        } else {
            Err(MyError::TcpStreamError("Error connecting  c client to server".to_string()).into())
        }
    }

    /// Se conecta con el server.
    /// Lee las consultas por stdin y las envía al server.
    /// Recibe la respuesta del server como Envelope y la imprime por pantalla.
    pub fn handle_connection(&self) -> Result<(), MyError> {
        let mut stream = match TcpStream::connect(&self.server_ip) {
            Ok(s) => s,
            Err(e) => {
                return Err(MyError::TcpStreamError(e.to_string()));
            }
        };
        let mut stream_clone = match stream.try_clone() {
            Ok(clone) => clone,
            Err(e) => {
                return Err(MyError::TcpStreamError(e.to_string()));
            }
        };

        let logger = Arc::clone(&self.logger);
        thread::spawn(move || {
            let stdin = io::stdin();
            loop {
                let mut input = String::new();
                if stdin.lock().read_line(&mut input).is_ok() {
                    let _ = Client::send_message(&mut stream_clone, input, config::QUORUM, logger.clone());
                }
            }
        });

        loop {
            let logger_clone = Arc::clone(&self.logger);
            match Envelope::from_stream(&mut stream, logger_clone) {
                Ok(env) => match env.body {
                    CqlMessage::Result(r) => match r.body {
                        ResultBody::Rows(r) => {
                            for row in r.rows_content {
                                println!("{}", row);
                            }
                        }
                        ResultBody::Empty(_) => {
                            println!();
                        }
                        ResultBody::Void(_) => {
                            println!("Error: Error en la consulta ingresada");
                        }
                    },
                    CqlMessage::Error(_) => {
                        println!("Error: Paso algo inesperado");
                    }
                    _ => {}
                },
                Err(e) => {
                    println!("Error al leer el env: {:?}", e);
                    break;
                }
            }
        }
        Ok(())
    }

    /// Envía la consulta por el stream al server.
    /// El mensaje que se envía es de tipo QueryMessage.
    pub fn send_message(stream: &mut TcpStream, message: String, consistency: u16, logger: Arc<RwLock<Logger>>) -> Result<(), MyError> {
        if !message.is_empty() {
            let msg: QueryMessage = QueryMessage {
                query: message,
                consistency, //config::QUORUM or ONE
            };
            let _ = msg.send_through(stream, logger);
        }
        Ok(())
    }

    /// Se lee la respuesta del server por el stream y se devuelve el resultado.
    pub fn read_response(stream: &mut TcpStream, logger:Arc<RwLock<Logger>>) -> Result<Vec<String>, MyError> {
        loop {
            let logger_clone = Arc::clone(&logger);
            match Envelope::from_stream(stream, logger_clone) {
                Ok(env) => match env.body {
                    CqlMessage::Result(result) => match result.body {
                        ResultBody::Void(_) => {
                            return Ok(vec!["".to_string()]);
                        }
                        ResultBody::Rows(rows) => {
                            return Ok(rows.rows_content);
                        }
                        ResultBody::Empty(_) => {
                            return Ok(vec!["".to_string()]);
                        }
                    },
                    _ => {
                        continue;
                    }
                },
                Err(e) => {
                    return Err(MyError::Error(e.to_string()).into());
                }
            }
        }
    }

    /// Devolvemos el stream con el primer nodo conectado que se encuentre.
    /// En caso de no haber nodos conectados se devuleve un error.
    pub fn connect_with_node() -> Result<TcpStream, MyError> {
        for number_node in 1..=RING_SIZE {
            let mut client = Client::initialize(number_node.to_string())?;
            if let Ok(stream) = client.connect() {
                return Ok(stream);
            }
        }
        Err(MyError::TcpStreamError("Error in TcpStream".to_string()).into())
    }
}

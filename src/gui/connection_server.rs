use std::{
    net::TcpStream,
    sync::mpsc::{self, Receiver, Sender},
    thread,
};

use crate::{client_cql::Client, my_errors::MyError, simulators::airport::Airport};

#[derive(Debug)]
pub struct ConnectionServer {
    pub stream: Option<TcpStream>,
    pub channels: (Sender<Vec<String>>, Receiver<Vec<String>>),
}

impl ConnectionServer {
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

    pub fn get_channel_to_receive(&self) -> &Receiver<Vec<String>> {
        &self.channels.1
    }

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

        let _ = thread::spawn({
            move || {
                if let Ok(origin_flights) =
                    airport.get_origin_flights_from(&mut Some(stream_server), &date)
                {
                    let _ = tx.send(origin_flights);
                }
                if let Ok(return_flights) =
                    airport.get_flights_that_return_to(&mut Some(stream_server_2), &date)
                {
                    let _ = tx.send(return_flights);
                }
            }
        });
        println!("Ok send flights");
        Ok(())
    }

    pub fn make_query(&self, query: String) -> Result<(), MyError> {
        println!("\n- Query: {}\n", query);
        let mut stream_server: TcpStream = if let Ok(stream) = self.get_connection_server() {
            stream
        } else {
            return Err(MyError::TcpStreamError("Error in stream".to_string()));
        };

        let tx = self.channels.0.clone();

        let _ = thread::spawn({
            move || {
                if Client::send_message(&mut stream_server, query).is_ok() {
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

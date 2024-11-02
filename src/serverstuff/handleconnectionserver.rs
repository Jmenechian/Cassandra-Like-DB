// Handler que se encarga de manejar la conexion del servidor con si mismo.
//
// La funcion principal es `handle_connection_server` que recibe un nodo, un stream y un booleano.
// El resto son funciones auxiliares.

use crate::cql::query::Query;
use crate::messagestuff::message_server::resultservermessage::IntoResultServerMessage;
use crate::messagestuff::message_server::resultvoidservermsg::ResultVoidServerMessage;
use crate::messagestuff::{
    message::*,
    message_server::{
        queryservermessage::QueryServerMessage, resultrowsevermsg::ResultRowsServerMessage,
        serverpacket::Packet, serverpacket::ServerMessage,
    },
};
use crate::my_errors::*;
use crate::nodo::Node;
use std::net::TcpStream;
use std::sync::{Arc, RwLock};

fn handle_message_query(
    node: Arc<RwLock<Node>>,
    query: QueryServerMessage,
    stream: &mut TcpStream,
    show: bool,
) -> Result<(), MyError> {
    let node = match node.read() {
        Ok(n) => n,
        Err(e) => {
            let _error = format!("Error al tratar de usar el nodo: {:?}", e);
            if show {
                println!("S   {:?}", _error);
            }
            return Err(MyError::LockingError(_error));
        }
    };
    let mut query = Query::new(query.query.clone(), node.server_ip.clone());
    let msg = match query.execute() {
        Ok(s) => {
            if show {
                println!("RESULTADO, TABLA: {:?}", s);
            }
            ResultRowsServerMessage {
                rows_count: s.len() as i32,
                rows_content: s,
            }
            .into_result_server_message()
        }
        Err(e) => {
            println!("Error en query: {:?}", e);
            ResultVoidServerMessage.into_result_server_message()
        }
    };
    let _ = msg.send_server_through(stream);

    Ok(())
}

pub fn handle_connection_server(
    node: Arc<RwLock<Node>>,
    stream: &mut TcpStream,
    show: bool,
) -> Result<(), MyError> {
    match Packet::from_stream(stream) {
        Ok(packet) => {
            if show {
                println!("S   El packet recibido es: {:?}", packet);
            }
            match packet.body {
                ServerMessage::Query(query) => {
                    handle_message_query(node.clone(), query, stream, show)
                }
                _ => {
                    let _error = "Error entre server y servers".to_string();
                    if show {
                        println!("S   {:?}", _error);
                    }
                    return Err(MyError::ServerMessageError(_error));
                }
            }
        }
        Err(_e) => Err(_e),
    }
}

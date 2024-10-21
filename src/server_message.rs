// Nodos se comunican entre si usando ServerMessage
// Los mensajes comienzan con REQUEST, RESULT o GOSSIP dependiendo del tipo de mensaje
//
// REQUEST: lo manda el nodo que habla con el cliente a los nodos correspondientes con la consulta
// RESULT: lo manda el nodo que devuelve la tabla al nodo que hablo con el cliente
// RECEIVED: para cumplir con TCP
// GOSSIP: para informar sobre nuevos nodos y nodos caidos
// TEST: para debuggear
//
// Ej: "REQUEST:<nodo.server_ip>:SELECT * FROM table" -> el nodo siendo el que primero habla con el cliente
// Ej: "RESULT:"<nodo.server_ip>:<tabla resultado>"  -> el nodo siendo el que devuelve la tabla
// Ej: "GOSSIP: ..........." -> a definir
// Ej: "RECEIVED:<nodo.server_ip>:SELECT * FROM table" -> el nodo siendo el que recibe la query del nodo principal"
// Ej: "TEST:<cualquier cosa>" o "<cualquier cosa>" -> no hace falta que empiece con TEST

use crate::my_errors::MyError;

#[derive(Debug, PartialEq, Eq)]
pub enum MessageType {
    Request,
    Result,
    Gossip,
    Received,
    Test, //////////// usar para debuggear
}

#[derive(Debug)]
pub struct ServerMessage {
    pub message_type: MessageType,
    pub message: String,
    pub original_sender_ip: String,
}

impl ServerMessage {
    pub fn new(message: String) -> Result<ServerMessage, MyError> {
        let message_type = ServerMessage::assert_message_type(&message);
        let (original_sender_ip, message) =
            ServerMessage::get_clean_message(&message, &message_type)?;

        Ok(ServerMessage {
            message_type,
            message,
            original_sender_ip,
        })
    }

    pub fn assert_message_type(message: &str) -> MessageType {
        if message.starts_with("REQUEST") {
            MessageType::Request
        } else if message.starts_with("RESULT") {
            MessageType::Result
        } else if message.starts_with("GOSSIP") {
            MessageType::Gossip
        } else if message.starts_with("RECEIVED") {
            MessageType::Received
        } else {
            MessageType::Test
        }
    }

    pub fn get_clean_message(
        message: &str,
        message_type: &MessageType,
    ) -> Result<(String, String), MyError> {
        match message_type {
            MessageType::Test => return Ok(("".to_string(), message.to_string())),
            MessageType::Gossip => return Ok(("".to_string(), "".to_string())), // falta implementar
            _ => {}
        }

        let message = message.trim().replace("\n", "").replace("\r", "");
        let parts: Vec<&str> = message.split(":").collect();

        if parts.len() < 3 {
            return Err(MyError::ServerMessageError(format!(
                "Invalid server-server message format: {}",
                message
            )));
        }

        let original_ip = parts[1..3].join(":");
        let cleaned_message = parts[3..].join(":");

        Ok((original_ip, cleaned_message))
    }
}

// Nodo 127.0.0.1:7001 recibe una consulta del cliente
// Este le manda la consulta a los nodos correspondientes
#[test]
fn test_request_server_message() {
    let message = "REQUEST:127.0.0.1:7001:SELECT * FROM table".to_string();
    let server_message = ServerMessage::new(message).unwrap();
    assert_eq!(server_message.message_type, MessageType::Request);
    assert_eq!(server_message.message, "SELECT * FROM table".to_string());
    assert_eq!(
        server_message.original_sender_ip,
        "127.0.0.1:7001".to_string()
    );
}

// Nodo recibe confirmacion de nodo 127.0.0.1:7002 que su mensaje fue recibido
#[test]
fn test_receive_server_message() {
    let message = "RECEIVED:127.0.0.1:7002:SELECT * FROM table".to_string();
    let server_message = ServerMessage::new(message).unwrap();
    assert_eq!(server_message.message_type, MessageType::Received);
    assert_eq!(server_message.message, "SELECT * FROM table".to_string());
    assert_eq!(
        server_message.original_sender_ip,
        "127.0.0.1:7002".to_string()
    );
}

// Nodo 127.0.0.1:7002 recibe una consulta de algun nodo
// Este le manda el resultado de la consulta al nodo que le hablo
#[test]
fn test_result_server_message() {
    let message = "RESULT:127.0.0.1:7002:<resultados tabla>".to_string();
    let server_message = ServerMessage::new(message).unwrap();
    assert_eq!(server_message.message_type, MessageType::Result);
    assert_eq!(server_message.message, "<resultados tabla>".to_string());
    assert_eq!(
        server_message.original_sender_ip,
        "127.0.0.1:7002".to_string()
    );
}

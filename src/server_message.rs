// Nodos se comunican entre si usando ServerMessage
// Los mensajes comienzan con REQUEST, RESULT o GOSSIP dependiendo del tipo de mensaje
//
// REQUEST: lo manda el nodo que habla con el cliente a los nodos correspondientes
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

use std::error::Error;

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
    pub fn new(message: String) -> Result<ServerMessage, Box<dyn Error>> {
        let message_type = ServerMessage::assert_message_type(&message);
        let (original_sender_ip, message) = ServerMessage::get_clean_message(&message, &message_type);

        Ok(ServerMessage {
            message_type,
            message,
            original_sender_ip,
        })
    }

    pub fn assert_message_type(message: &str) -> MessageType {
        if message.starts_with("REQUEST") {
            return MessageType::Request;
        } else if message.starts_with("RESULT") {
            return MessageType::Result;
        } else if message.starts_with("GOSSIP") {
            return MessageType::Gossip;
        } else if message.starts_with("RECEIVED") {
            return MessageType::Received;
        } else {
            return MessageType::Test;
        }
    }

    pub fn get_clean_message(message: &String, message_type: &MessageType) -> (String,String) {
        match message_type {
            MessageType::Test => return ("".to_string(), "".to_string()),
            MessageType::Gossip => return ("".to_string(), "".to_string()), // falta implementar
            _ => {}
        }

        let message = message.trim().replace("\n", "").replace("\r", "");
        let parts: Vec<&str> = message.split(":").collect();
        let original_ip = parts[1..3].join(":");
        let cleaned_message = parts[3..].join(":");

        (original_ip, cleaned_message)
    }
}
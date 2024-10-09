// Nodos se comunican entre si usando ServerMessage
// Los mensajes comienzan con REQUEST, RESULT o GOSSIP dependiendo del tipo de mensaje
// Ej: "REQUEST:SELECT * FROM tabla WHERE condicion"
// Ej: "GOSSIP:puerto1 nuevo_nodo"

use std::error::Error;

#[derive(Debug)]
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
}

impl ServerMessage {
    pub fn new(message: String) -> Result<ServerMessage, Box<dyn Error>> {
        let message_type = ServerMessage::assert_message_type(&message);
        let message = ServerMessage::get_clean_message(&message);

        Ok(ServerMessage {
            message_type,
            message,
        })
    }

    pub fn assert_message_type(message: &String) -> MessageType {
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

    pub fn get_clean_message(message: &String) -> String {
        let message = message.trim().replace("\n", "").replace("\r", "");
        let parts: Vec<&str> = message.split(":").collect();
        let cleaned_message = parts[1..].join(":");

        cleaned_message
    }
}
use crate::messagestuff::message_server::{resultservermessage::*, serverpacket::*};

#[derive(Debug, PartialEq)]
pub struct ResultEmptyServerMessage;

impl IntoResultServerMessage for ResultEmptyServerMessage {
    fn into_result_server_message(self) -> ResultServerMessage {
        ResultServerMessage {
            kind: ResultServerKind::Empty,
            body: ResultServerBody::Empty(self),
        }
    }
}

impl IntoPacket for ResultEmptyServerMessage {
    fn into_packet(self) -> Packet {
        let result = self.into_result_server_message();
        result.into_packet()
    }
}

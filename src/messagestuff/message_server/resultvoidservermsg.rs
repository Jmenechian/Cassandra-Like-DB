use crate::messagestuff::message_server::{resultservermessage::*, serverpacket::*};

#[derive(Debug, PartialEq)]
pub struct ResultVoidServerMessage;

impl IntoResultServerMessage for ResultVoidServerMessage {
    fn into_result_server_message(self) -> ResultServerMessage {
        ResultServerMessage {
            kind: ResultServerKind::Void,
            body: ResultServerBody::Void(self),
        }
    }
}

impl IntoPacket for ResultVoidServerMessage {
    fn into_packet(self) -> Packet {
        let result = self.into_result_server_message();
        result.into_packet()
    }
}

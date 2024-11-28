// Representa un resultado vacio a una consulta

use crate::messagestuff::message_cql::{cqlenvelope::*, resultmessage::*};

#[derive(Debug, PartialEq)]
pub struct ResultVoidMessage;

impl IntoResultMessage for ResultVoidMessage {
    fn into_result_message(self) -> ResultMessage {
        ResultMessage {
            kind: ResultKind::Void,
            body: ResultBody::Void(self),
        }
    }
}

impl IntoEnvelope for ResultVoidMessage {
    fn into_envelope(self, cql_stream: u16) -> Envelope {
        let result = self.into_result_message();
        result.into_envelope(cql_stream)
    }
}

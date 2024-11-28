// Representa un resultado vacio a una consulta

use crate::messagestuff::message_cql::{cqlenvelope::*, resultmessage::*};

#[derive(Debug, PartialEq)]
pub struct ResultEmptyMessage;

impl IntoResultMessage for ResultEmptyMessage {
    fn into_result_message(self) -> ResultMessage {
        ResultMessage {
            kind: ResultKind::Empty,
            body: ResultBody::Empty(self),
        }
    }
}

impl IntoEnvelope for ResultEmptyMessage {
    fn into_envelope(self) -> Envelope {
        let result = self.into_result_message();
        result.into_envelope()
    }
}

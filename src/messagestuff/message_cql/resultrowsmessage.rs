// Representa a una fila de la tabla de resultado
// del tipo ResultMessage
use crate::errors::my_errors::MyError;
use crate::messagestuff::message::*;
use crate::messagestuff::message_cql::{cqlenvelope::*, resultmessage::*};

#[derive(Debug, PartialEq)]
pub struct ResultRowsMessage {
    pub rows_count: i32,
    pub rows_content: Vec<String>,
}

impl Serialize for ResultRowsMessage {
    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&int_to_bytes(self.rows_count));
        for row in &self.rows_content {
            bytes.extend_from_slice(&string_to_bytes(row.clone()));
        }
        bytes
    }

    fn from_bytes(bytes: Vec<u8>) -> Result<(Self, Vec<u8>), MyError> {
        let (_rows_count, resto) = int_from_bytes(bytes)?;
        let mut _bytes = resto;
        let mut _rows_content: Vec<String> = Vec::new();
        for _ in 0.._rows_count {
            let (_row, resto) = string_from_bytes(_bytes)?;
            _rows_content.push(_row);
            _bytes = resto;
        }
        Ok((
            ResultRowsMessage {
                rows_count: _rows_count,
                rows_content: _rows_content,
            },
            _bytes,
        ))
    }
}

impl IntoResultMessage for ResultRowsMessage {
    fn into_result_message(self) -> ResultMessage {
        ResultMessage {
            kind: ResultKind::Rows,
            body: ResultBody::Rows(self),
        }
    }
}

impl IntoEnvelope for ResultRowsMessage {
    fn into_envelope(self, cql_stream: u16) -> Envelope {
        let result = self.into_result_message();
        result.into_envelope(cql_stream)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_result_rows_message() -> Result<(), MyError> {
        let expected_result: ResultRowsMessage = ResultRowsMessage {
            rows_count: 1 as i32,
            rows_content: vec!["hola".to_string()],
        };
        let bytes: Vec<u8> = expected_result.to_bytes();
        let (_result, _) = ResultRowsMessage::from_bytes(bytes)?;
        Ok(assert_eq!(_result, expected_result))
    }

    #[test]
    fn test_result_rows_message_2() -> Result<(), MyError> {
        let expected_result: ResultRowsMessage = ResultRowsMessage {
            rows_count: 2 as i32,
            rows_content: vec!["hola".to_string(), "chau".to_string()],
        };
        let bytes: Vec<u8> = expected_result.to_bytes();
        let (_result, _) = ResultRowsMessage::from_bytes(bytes)?;
        Ok(assert_eq!(_result, expected_result))
    }

    #[test]
    fn test_result_rows_message_3() -> Result<(), MyError> {
        let expected_result: ResultRowsMessage = ResultRowsMessage {
            rows_count: 2 as i32,
            rows_content: vec!["hola".to_string(), "chau".to_string()],
        };
        let mut bytes: Vec<u8> = expected_result.to_bytes();
        bytes.pop();
        if let Err(e) = ResultRowsMessage::from_bytes(bytes) {
            let error = e.to_string();
            return Ok(assert_eq!(
                error,
                "[TCP_STREAM_ERROR]: [No hay suficientes bytes prar leer el <String>]"
            ));
        }
        Ok(())
    }
    #[test]
    fn test_result_rows_message_4() -> Result<(), MyError> {
        let expected_result: ResultRowsMessage = ResultRowsMessage {
            rows_count: 3 as i32,
            rows_content: vec!["hola".to_string(), "chau".to_string()],
        };
        let mut bytes: Vec<u8> = expected_result.to_bytes();
        bytes.pop();
        if let Err(e) = ResultRowsMessage::from_bytes(bytes) {
            return Ok(assert_eq!(
                e,
                MyError::TcpStreamError(
                    "No hay suficientes bytes prar leer el <String>".to_string()
                )
            ));
        }
        Ok(())
    }
}

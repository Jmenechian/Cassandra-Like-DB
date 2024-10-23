use std::error::Error;
use std::fmt;
use std::io;

#[derive(Debug)]
pub enum MyError {
    Error(String),
    InvalidTable(String),
    InvalidColumn(String),
    InvalidSyntax(String),
    TestError(String),
    ThreadError(String),
    TcpStreamError(String),
    TcpListenerError(String),
    ServerMessageError(String),
    SendingMessageError(String),
    LockingError(String),
    FileError(String),
}

impl fmt::Display for MyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            MyError::Error(ref msg) => write!(f, "[ERROR]: [{}]", msg),
            MyError::InvalidTable(ref msg) => write!(f, "[INVALID_TABLE]: [{}]", msg),
            MyError::InvalidColumn(ref msg) => write!(f, "[INVALID_COLUMN]: [{}]", msg),
            MyError::InvalidSyntax(ref msg) => write!(f, "[INVLAID_SYNTAX]: [{}]", msg),
            MyError::TestError(ref msg) => write!(f, "[TEST_ERROR]: [{}]", msg),
            MyError::ThreadError(ref msg) => write!(f, "[THREAD_ERROR]: [{}]", msg),
            MyError::TcpStreamError(ref msg) => write!(f, "[TCP_STREAM_ERROR]: [{}]", msg),
            MyError::TcpListenerError(ref msg) => write!(f, "[TCP_LISTENER_ERROR]: [{}]", msg),
            MyError::ServerMessageError(ref msg) => write!(f, "[SERVER_MESSAGE_ERROR]: [{}]", msg),
            MyError::SendingMessageError(ref msg) => {
                write!(f, "[SENDING_MESSAGE_ERROR]: [{}]", msg)
            }
            MyError::LockingError(ref msg) => write!(f, "[LOCKING_ERROR]: [{}]", msg),
            MyError::FileError(ref msg) => write!(f, "[FILE_ERROR]: [{}]", msg),
        }
    }
}

impl From<MyError> for io::Error {
    fn from(error: MyError) -> io::Error {
        io::Error::new(io::ErrorKind::Other, error.to_string())
    }
}

impl PartialEq for MyError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (MyError::Error(a), MyError::Error(b)) => a == b,
            (MyError::InvalidTable(a), MyError::InvalidTable(b)) => a == b,
            (MyError::InvalidColumn(a), MyError::InvalidColumn(b)) => a == b,
            (MyError::InvalidSyntax(a), MyError::InvalidSyntax(b)) => a == b,
            (MyError::TestError(a), MyError::TestError(b)) => a == b,
            (MyError::ThreadError(a), MyError::ThreadError(b)) => a == b,
            (MyError::TcpStreamError(a), MyError::TcpStreamError(b)) => a == b,
            (MyError::TcpListenerError(a), MyError::TcpListenerError(b)) => a == b,
            (MyError::ServerMessageError(a), MyError::ServerMessageError(b)) => a == b,
            (MyError::SendingMessageError(a), MyError::SendingMessageError(b)) => a == b,
            (MyError::LockingError(a), MyError::LockingError(b)) => a == b,
            (MyError::FileError(a), MyError::FileError(b)) => a == b,
            _ => false,
        }
    }
}

impl Error for MyError {}

// Para joinear threads y convertir sus errores
pub fn join_thread(thread: std::thread::JoinHandle<()>) -> Result<(), MyError> {
    match thread.join() {
        Ok(_) => Ok(()),
        Err(e) => {
            let err = if let Some(s) = e.downcast_ref::<&str>() {
                MyError::ThreadError(s.to_string())
            } else if let Some(s) = e.downcast_ref::<String>() {
                MyError::ThreadError(s.clone())
            } else {
                MyError::ThreadError("Thread panicked with unknown error".to_string())
            };
            Err(err)
        }
    }
}

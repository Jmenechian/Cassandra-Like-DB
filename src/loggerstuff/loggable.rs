use crate::errors::my_errors::MyError;

/// Trait que usan los Nodos y el Cliente para poder escribir los logs de los Envelopes.
pub trait Loggable {
    fn write_log(&self, message: &str) -> Result<(), MyError>;
}
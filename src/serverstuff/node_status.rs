use crate::config;
use crate::errors::my_errors::MyError;

#[derive(Debug, Clone, PartialEq, Copy)]
#[repr(u8)]
pub enum Status {
    Connected = config::CONNECTED,
    Disconnected = config::DISCONNECTED,
    Disengaged = config::DISENGAGED,
}

impl Status {
    pub fn from_u8(value: u8) -> Result<Self, MyError> {
        match value {
            config::CONNECTED => Ok(Status::Connected),
            config::DISCONNECTED => Ok(Status::Disconnected),
            config::DISENGAGED => Ok(Status::Disengaged),
            _ => Err(MyError::Error("Invalid Status".to_string())),
        }
    }
}

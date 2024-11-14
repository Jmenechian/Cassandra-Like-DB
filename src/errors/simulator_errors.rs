use std::fmt;

#[derive(Debug)]
pub enum SimulatorErrors {
    Error(String),
    FlightIsAlreadySimulated(),
    UncreatedFlight(),
    InputError(),
    AllNodesDisconnected(),
}

impl fmt::Display for SimulatorErrors {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            SimulatorErrors::Error(ref msg) => write!(f, "ERROR: {}", msg),
            SimulatorErrors::FlightIsAlreadySimulated() => {
                write!(f, "ERROR: El vuelo ya está siendo simulado.")
            }
            SimulatorErrors::UncreatedFlight() => {
                write!(f, "ERROR: No se pudo crear el vuelo de simulación.")
            }
            SimulatorErrors::InputError() => {
                write!(f, "ERROR: No se pudo leer el input correctamente.")
            }
            SimulatorErrors::AllNodesDisconnected() => {
                write!(f, "ERROR: No se encontró un nodo conectado.")
            }
        }
    }
}

use aerolineas_rusticas::simulators::flight_simulator::FlightSimulator;

/// Ejecución del simulador de vuelos.
/// Se crean vuelos en diferentes momentos del tiempo y se actualizan los valores de sus variables, imitando el comportamiento de un
/// avión que despega en un aeropuerto de origen, llega a velocidad crucero y aterriza en el aeropuerto de destino.
fn main() {
    if let Err(error) = FlightSimulator::run() {
        println!("{}", error);
    }
}

use std::env;

use aerolineas_rusticas::simulators::flight_simulator::FlightSimulator;

/// Ejecución del simulador de vuelos.
/// Se crean vuelos en diferentes momentos del tiempo y se actualizan los valores de sus variables, imitando el comportamiento de un
/// avión que despega en un aeropuerto de origen, llega a velocidad crucero y aterriza en el aeropuerto de destino.
fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        return;
    }
    let ports_file = args[1].to_string();

    if let Err(error) = FlightSimulator::run(ports_file) {
        println!("{}", error);
    }
}

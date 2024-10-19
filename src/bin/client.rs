use aerolineas_rusticas::clientstuff::client_cql::Client;
use std::env;

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Uso: {} <archivo_de_puertos>", args[0]);
        std::process::exit(1);
    }
    let port = args[1].to_string();

    let client = Client::initialize(port);

    println!("Cliente: {:?}", client);

    if let Err(_) = client.handle_connection_2() { //solo manda no escucha
        println!("Error");
    }
    Ok(())
}

// cuando escuche: ->esto ya anda 
//     agarras solo message.message en nodo y podes procesar la query

// aparte:
//     seguir con el protocolo
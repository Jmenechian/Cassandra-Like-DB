use std::io::{self, stdin, stdout, Write};

/// Se le preguntan por consola al usuario los datos del vuelo.
pub fn ask_user_flight_values() -> Result<Vec<String>, io::Error> {
    let requests = vec![
        "numero",
        "aeropuerto de origen",
        "aeropuerto de destino",
        "velocidad promedio estimada",
    ];
    let mut result: Vec<String> = vec![];
    for request in requests {
        let input = get_input(request)?;
        result.push(input);
    }

    Ok(result)
}

fn get_input(request: &str) -> Result<String, io::Error> {
    println!("Introduce un valor de {} del vuelo: ", request);
    stdout().flush()?;

    let mut input = String::new();

    stdin().read_line(&mut input)?;

    input = input.trim().to_string();

    if !correct_input(request, &input) {
        println!("El valor ingresado no es valido. \nEl numero de vuelo debe ser un entero positivo y la velocidad un flotante positivo.");
        return get_input(request);
    }

    Ok(input)
}

fn correct_input(request: &str, input: &str) -> bool {
    match request {
        "numero" => {
            if input.parse::<usize>().is_err() {
                return false;
            }
        }
        "velocidad promedio estimada" => {
            if input.parse::<f32>().is_err() {
                return false;
            } else if let Ok(value) = input.parse::<f32>() {
                if value < 0.0 {
                    return false;
                }
            }
        }
        _ => return true,
    }
    true
}

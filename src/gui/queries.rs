/// Se devuelve la query para insertar un vuelo en la tabla de origen, dependiendo de los valores ingresados.
pub fn get_origin_insert_query(
    origin_airport: &String,
    destination_airport: &String,
    date: &String,
    flight: &String,
    status: &String,
) -> String {
    format!(
        "INSERT INTO origin (airport, destination, date, flight, status) VALUES ('{}', '{}', '{}', '{}', '{}')",
        origin_airport, destination_airport, date, flight, status
    )
}

/// Se devuelve la query para insertar un vuelo en la tabla de destino, dependiendo de los valores ingresados.
pub fn get_destination_insert_query(
    origin_airport: &String,
    destination_airport: &String,
    date: &String,
    flight: &String,
    status: &String,
) -> String {
    format!(
        "INSERT INTO destination (airport, origin, date, flight, status) VALUES ('{}', '{}', '{}', '{}', '{}')",
        destination_airport, origin_airport, date, flight, status
    )
}

/// Se devuelve la query para actualizar el status de un vuelo en la tabla de origen, dependiendo de los valores ingresados.
pub fn get_origin_update_query(
    origin_airport: &String,
    date: &String,
    flight: &String,
    status: &String,
) -> String {
    format!(
        "UPDATE origin SET status = '{}' WHERE date = '{}' AND airport = '{}' AND flight = '{}'",
        status, date, origin_airport, flight
    )
}

/// Se devuelve la query para actualizar el status de un vuelo en la tabla de destino, dependiendo de los valores ingresados.
pub fn get_destination_update_query(
    destination_airport: &String,
    date: &String,
    flight: &String,
    status: &String,
) -> String {
    format!(
        "UPDATE destination SET status = '{}' WHERE date = '{}' AND airport = '{}' AND flight = '{}'",
        status, date, destination_airport, flight
    )
}

/// Se devuelve la query para borrar un vuelo en la tabla de origen, dependiendo de los valores ingresados.
pub fn get_origin_delete_query(origin_airport: &String, date: &String, flight: &String) -> String {
    format!(
        "DELETE FROM origin WHERE date = {} AND airport = {} AND flight = {}",
        date, origin_airport, flight
    )
}

/// Se devuelve la query para borrar un vuelo en la tabla de origen, dependiendo de los valores ingresados.
pub fn get_destination_delete_query(destination_airport: &String, date: &String, flight: &String) -> String {
    format!(
        "DELETE FROM destination WHERE date = {} AND airport = {} AND flight = {}",
        date, destination_airport, flight
    )
}

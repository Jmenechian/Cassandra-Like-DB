/// Query para borrar una partición determianda de una una tabla.
pub fn get_partition_delete_query(table: &str, airport: &str, date: &str) -> String {
    format!(
        "DELETE FROM {} WHERE airport = {} AND date = {}",
        table, airport, date
    )
}

/// Query para insertar nuevos registros.
pub fn get_insert_query(table: &str, headers: Vec<String>, line: Vec<String>) -> String {
    format!(
        "INSERT INTO {} ({}) VALUES ({})",
        table,
        headers.join(", "),
        line.join(", ")
    )
}

/// Query para crear la tabla indicada.
pub fn get_create_query(table: &str) -> String {
    match table {
        "origin" => "CREATE TABLE origin (airport STR, destination STR, date DATETIME, flight STR, status STR, timestamp TIMESTAMP, PRIMARY KEY (airport, date, flight)) WITH CLUSTERING ORDER BY (timestamp ASC)".to_string(),
        "destination" => "CREATE TABLE destination (airport STR, origin STR, date DATETIME, flight STR, status STR, timestamp TIMESTAMP, PRIMARY KEY (airport, date, flight)) WITH CLUSTERING ORDER BY (timestamp ASC)".to_string(),
        "" => "CREATE TABLE flights_in_progress (flight STR, airport STR, destination STR, status STR, date DATETIME, speed STR, latitude STR, longitude STR, height STR, timestamp TIMESTAMP, PRIMARY KEY (number, origin, date) WITH CLUSTERING ORDER BY (timestamp ASC)".to_string(),
        _ => "".to_string()
    }
}

/// Query para borrar una partición determianda de una una tabla.
pub fn get_partition_delete_query(table: &str, airport: &str, date: &str) -> String {
    let table_name = get_table_name_from_path(table);
    format!(
        "DELETE FROM {} WHERE airport = {} AND date = {}",
        table_name, airport, date
    )
}

/// Query para insertar nuevos registros.
pub fn get_insert_query(table: &str, headers: &mut Vec<String>, line: &mut Vec<String>) -> String {
    let table_name = get_table_name_from_path(table);
    headers.pop();
    line.pop();
    format!(
        "INSERT INTO {} ({}) VALUES ({})",
        table_name,
        headers.join(", "),
        line.join(", ")
    )
}

/// Query para crear la tabla indicada.
pub fn get_create_query(table: &str) -> String {
    match table {
        "origin" => "CREATE TABLE origin (airport STR, destination STR, date DATETIME, flight STR, status STR, timestamp TIMESTAMP, PRIMARY KEY (airport, date, flight)) WITH CLUSTERING ORDER BY (timestamp ASC)".to_string(),
        "destination" => "CREATE TABLE destination (airport STR, origin STR, date DATETIME, flight STR, status STR, timestamp TIMESTAMP, PRIMARY KEY (airport, date, flight)) WITH CLUSTERING ORDER BY (timestamp ASC)".to_string(),
        "flights_in_progress" => "CREATE TABLE flights_in_progress (flight STR, airport STR, destination STR, status STR, date DATETIME, speed STR, latitude STR, longitude STR, height STR, timestamp TIMESTAMP, PRIMARY KEY (number, origin, date) WITH CLUSTERING ORDER BY (timestamp ASC)".to_string(),
        _ => "".to_string()
    }
}

fn get_table_name_from_path(table: &str) -> String {
    if table.contains("origin") {
        "origin".to_string()
    } else if table.contains("destination") {
        "destination".to_string()
    } else if table.contains("flights_in_progress") {
        "flights_in_progress".to_string()
    } else {
        "".to_string()
    }
}

pub static PORTS_FILE: &str = "puertos.txt";

pub static POOL_NUMBER: usize = 5;

pub static STRONG_CONSISTENCY: usize = 1; //en realidad deberia ser 2 //si RF es 3 esto cumple Quorum

pub static SERVER_PORT: &str = "7000";

pub static CLIENT_PORT: &str = "9042";

pub static N_WORKERS: usize = 10; //Cantidad de thread que lanza el thread pool. (Nos dijeron que usemos 10).

pub static RF: usize = 3; // Valor del Replication Factor usado en Consistent Hashing.

pub static TABLE_PATH: &str = "src/tables";

pub static FILE_AIRPORT: &str = "/Users/martinalarosa/Desktop/Taller1/24C2-Ferris/src/tables/airports.csv";

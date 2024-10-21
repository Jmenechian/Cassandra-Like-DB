pub static PORTS_FILE: &str = "puertos.txt";

pub static POOL_NUMBER: usize = 5;

pub static STRONG_CONSISTENCY: usize = 1; //en realidad deberia ser 2 //si RF es 3 esto cumple Quorum

pub static SERVER_PORT: &str = "7000";

pub static CLIENT_PORT: &str = "9042";

pub static N_WORKERS: usize = 10; //Cantidad de thread que lanza el thread pool. (Nos dijeron que usemos 10).

pub static RF: usize = 3; // Valor del Replication Factor usado en Consistent Hashing.

pub static TABLE_PATH: &str = "src/tables";

// [relation.rs]
pub const EQUAL: i32 = 0;
pub const GREATER_THAN: i32 = 1;
pub const LESS_THAN: i32 = 2;
pub const NOT_NUMBER: i32 = 3;

// [message.rs]
pub const REQUEST: u8 = 0x03;
pub const QUORUM: u8 = 0x0004;
pub const QUERY: u8 = 0x07;
pub const RESPONSE: u8 = 0x83;
pub const RESULT: u8 = 0x08;
// const RESPONSE: u8 = 0x83;

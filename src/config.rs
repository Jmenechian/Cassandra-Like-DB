pub static PORTS_FILE: &str = "puertos.txt";

pub static KEY_FILE: &str = "src/encryption/config_aes/aes_key.bin";

pub static SERVER_PORT: &str = "7000";
pub static CLIENT_PORT: &str = "9042";

pub static TABLE_PATH: &str = "src/tables";
pub static TABLE_ORIGIN_NAME: &str = "origin";
pub static TABLE_DESTINATION_NAME: &str = "destination";
pub static TABLE_FLIGHT_SIMULATOR: &str = "flights_in_progress";

// Consistent Hashing:
pub static RF: usize = 3; // Valor del Replication Factor.
pub static RING_SIZE: usize = 8000; // Cantidad total de nodos servidor
pub static SEED_0: u32 = 0;
pub static SEED_1: u32 = 0x1F351F35; // Barker code * 2 -> 523.575.093
pub static SEED_2: u32 = 0xbc9f1d34; // 3.164.544.308

// SIMULADOR DE VUELOS:
pub static N_WORKERS: usize = 10; // Cantidad de workers para el thread pool.
pub static FLIGHT_STEP: f32 = 10.0; // Cantidad de pasos que da un avión hasta la mitad del vuelo.
pub static CRUISING_SPEED: f32 = 840.0; // Velocidad crucero de un avion, en km/h
pub static STEPS_AT_CRUISING_SPEED: f32 = 2.0; // Mitad de pasos en los que el avion viajara a la velocidad crucero.
pub static CRUISING_HEIGHT: f32 = 10000.0; // Altura crucero de 10mil metros.
pub static QUERIES_PATH: &str = "src/simulators/update_queries.csv";

//GUI:
pub static AIRPORTS_PATH: &str = "src/tables/airports.csv";
pub static MAP_PATH: &str = "src/gui/assets/map.png";
pub static PLANE_PATH: &str = "src/gui/assets/air-plane_{}.png";
pub static TABLE_USERS_NAME: &str = "users";

pub static RADIUS_CLICK_AIRPORT: f32 = 15.0; // Radio del botón
pub static RADIUS_PAINT_AIRPORT: f32 = 5.0; //
pub static PLANE_SIZE: f32 = 50.0;

//GUI OPTIONAL
pub static AIRPORT: i8 = 0;
pub static ORIGINDESTINATION: i8 = 1;
pub static AIRPORT2: i8 = 2;
pub static DATE: i8 = 3;
pub static FLIGHT: i8 = 4;
pub static STATUS: i8 = 5;
pub static SELECT: i8 = 6;
pub static UPDATE: i8 = 7;
pub static INSERT: i8 = 8;
pub static DELETE: i8 = 9;

// OPCODE
pub const QUERY: u8 = 0x07;
pub const ERROR: u8 = 0x00;
pub const RESULT: u8 = 0x08;

pub const GOSSIP: u8 = 0x11;
pub const READ_REPAIR: u8 = 0x12;

pub const CONNECTED: u8 = 0x20;
pub const DISCONNECTED: u8 = 0x21;

// [consistency]
pub const ONE: u16 = 0x0001;
pub const QUORUM: u16 = 0x0004;

// Result Kind
pub const VOID: i32 = 0x0001;
pub const ROWS: i32 = 0x0002;
pub const EMPTY: i32 = 0x0003;

// [relation.rs]
pub const EQUAL: i32 = 0;
pub const GREATER_THAN: i32 = 1;
pub const LESS_THAN: i32 = 2;
pub const NOT_NUMBER: i32 = 3;

// encriptacion at-rest
pub const ENCRIPT_DATA: bool = false; // Para testeo

// OPCODE
// 0x00    ERROR
// 0x01    STARTUP
// 0x02    READY
// 0x03    AUTHENTICATE
// 0x05    OPTIONS
// 0x06    SUPPORTED
// 0x07    QUERY
// 0x08    RESULT
// 0x09    PREPARE
// 0x0A    EXECUTE
// 0x0B    REGISTER
// 0x0C    EVENT
// 0x0D    BATCH
// 0x0E    AUTH_CHALLENGE
// 0x0F    AUTH_RESPONSE
// 0x10    AUTH_SUCCESS

// [consistency]  A consistency level specification. This is a [short]
// representing a consistency level with the following
// correspondance:
//   0x0000    ANY
//   0x0001    ONE
//   0x0002    TWO
//   0x0003    THREE
//   0x0004    QUORUM
//   0x0005    ALL
//   0x0006    LOCAL_QUORUM
//   0x0007    EACH_QUORUM
//   0x0008    SERIAL
//   0x0009    LOCAL_SERIAL
//   0x000A    LOCAL_ONE

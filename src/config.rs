pub static PORTS_FILE: &str = "puertos.txt";

pub static POOL_NUMBER: usize = 5;

pub static STRONG_CONSISTENCY: usize = 1; //en realidad deberia ser 2 //si RF es 3 esto cumple Quorum

pub static SERVER_PORT: &str = "7000";

pub static CLIENT_PORT: &str = "9042";
pub static RF: usize = 3; // Valor del Replication Factor usado en Consistent Hashing.

pub static TABLE_PATH: &str = "src/tables";

pub static N_WORKERS: usize = 10; //Nos dijeron que usemos 10.

// OPCODE
pub const QUERY: u8 = 0x07;
pub const ERROR: u8 = 0x00;
pub const RESULT: u8 = 0x08;

pub const GOSSIP: u8 = 0x11;

pub const CONNECTED: u8 = 0x20;
pub const DISCONNECTED: u8 = 0x21;

// [consistency]
pub const ONE: u16 = 0x0001;
pub const QUORUM: u16 = 0x0004;

// Result Kind
pub const VOID: i32 = 0x0001;
pub const ROWS: i32 = 0x0002;

// [relation.rs]
pub const EQUAL: i32 = 0;
pub const GREATER_THAN: i32 = 1;
pub const LESS_THAN: i32 = 2;
pub const NOT_NUMBER: i32 = 3;

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

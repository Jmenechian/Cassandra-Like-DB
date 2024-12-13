use crate::errors::my_errors::MyError;
use crate::utils::append_row;
use chrono::prelude::*;
use std::fs::{self, File};

#[derive(Debug)]
/// Estructura Logger que se encarga de guardar
/// los logs de los mensajes del nodouse std::io::BufWriter;
pub struct Logger {
    pub file: File,
    pub num_node: String,
}

impl Logger {
    /// Recibe el numero del nodo y devuelve un nuevo Logger
    pub fn new(num_node: String) -> Result<Logger, MyError> {
        let file = fs::OpenOptions::new()
            .write(true)
            .create(true)
            .append(true)
            .open(format!("src/loggerstuff/logs/log_{}.txt", num_node))
            .map_err(|_| MyError::FileError("Error al abrir archivo de logs".to_string()))?;

        Ok(Logger { file, num_node })
    }

    /// Escribe un mensaje en el archivo de logs
    pub fn write_log(&mut self, message: &str) -> Result<(), MyError> {
        let now = Local::now();
        let date = now.format("%Y-%m-%d %H:%M:%S").to_string();
        let log = format!("{} {}", date, message);

        println!("\n- {}", log);
        append_row(&mut self.file, &[log])?;
        Ok(())
    }
}

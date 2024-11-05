use super::airport::Airport;

#[derive(Debug)]
/// Estructura que representa un vuelo, con su nombre, el aeropuesto de origen y el de destino.
pub struct Flight {
    pub name: String,
    pub origin_airport: Airport,
    pub return_airport: Airport,
    pub status: String,
}

impl Flight {
    /// Se crea un nuevo vuelo con los datos indicados.
    pub fn new(
        name: String,
        origin_airport: Airport,
        return_airport: Airport,
        status: String,
    ) -> Self {
        Self {
            name,
            origin_airport,
            return_airport,
            status,
        }
    }
}

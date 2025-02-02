/// Tipo de los trabajos que deben realizar los threads del ThreadPool.
pub type Job = Box<dyn FnOnce() + Send + 'static>;

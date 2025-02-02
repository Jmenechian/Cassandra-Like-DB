/// Tipo de los trabajos que deben realizar los threads del ThreadPool.
pub(crate) type Job = Box<dyn FnOnce() + Send + 'static>;

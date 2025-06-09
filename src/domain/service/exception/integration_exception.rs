use std::fmt;

#[allow(dead_code)]
#[derive(Debug)]
pub enum IntegrationException {
    //ValidationError(String),
    PersistenceError(String),
    IntegrationError(String),
    NotFoundError(String),
    //Other(String),
}

impl fmt::Display for IntegrationException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            //IntegrationException::ValidationError(msg) => write!(f, "Validation error: {}", msg),
            IntegrationException::PersistenceError(msg) => write!(f, "Persistence error: {}", msg),
            IntegrationException::IntegrationError(msg) => write!(f, "Integration error: {}", msg),
            IntegrationException::NotFoundError(msg) => {
                write!(f, "Entity not found error: {}", msg)
            } //IntegrationException::Other(msg) => write!(f, "Other error: {}", msg),
        }
    }
}

impl std::error::Error for IntegrationException {}

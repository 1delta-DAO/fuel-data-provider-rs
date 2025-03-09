use std::fmt;

#[derive(Debug)]
pub enum DataException {
    BlockTimeEstimatorError(String),
    DataCleanupException(String),
}

impl fmt::Display for DataException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataException::BlockTimeEstimatorError(msg) => write!(f, "Persistence error: {}", msg),
            DataException::DataCleanupException(msg) => write!(f, "Data cleanup error: {}", msg),
        }
    }
}

impl std::error::Error for DataException {}
use std::error::Error;
use std::fmt::{self, Display};

#[derive(Debug)]
pub enum ValidationNotSetError {
    CreatedAt,
    Uuid,
    Fleet,
    ApiUrl,
    File,
}

impl Display for ValidationNotSetError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationNotSetError::CreatedAt => write!(f, "Created At is not set"),
            ValidationNotSetError::Uuid => write!(f, "UUID is not set"),
            ValidationNotSetError::Fleet => write!(f, "Fleet is not set"),
            ValidationNotSetError::ApiUrl => write!(f, "API URL is not set"),
            ValidationNotSetError::File => write!(f, "File is not set"),
        }
    }
}

impl Error for ValidationNotSetError {}

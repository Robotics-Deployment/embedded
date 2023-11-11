use std::error::Error;
use std::fmt::{self, Display};

#[derive(Debug)]
pub enum ValidationError {
    UuidNotSet,
    FleetNotSet,
    InterfaceNotSet,
    ApiUrlNotSet,
    FileNotSet,
}

impl Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationError::UuidNotSet => write!(f, "UUID is not set"),
            ValidationError::FleetNotSet => write!(f, "Fleet is not set"),
            ValidationError::InterfaceNotSet => write!(f, "Interface is not set"),
            ValidationError::ApiUrlNotSet => write!(f, "API URL is not set"),
            ValidationError::FileNotSet => write!(f, "File is not set"),
        }
    }
}

impl Error for ValidationError {}

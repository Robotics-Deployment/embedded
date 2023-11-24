use std::error::Error;
use std::fmt::{self, Display};

#[derive(Debug)]
pub enum NotSetError {
    CreatedAt,
    Uuid,
    Fleet,
    ApiUrl,
    File,
    PrivateKey,
    Address,
    Peers,
    WireguardFile,
}

impl Display for NotSetError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NotSetError::CreatedAt => write!(f, "Created At is not set"),
            NotSetError::Uuid => write!(f, "UUID is not set"),
            NotSetError::Fleet => write!(f, "Fleet is not set"),
            NotSetError::ApiUrl => write!(f, "API URL is not set"),
            NotSetError::File => write!(f, "File is not set"),
            NotSetError::PrivateKey => write!(f, "Private Key is not set"),
            NotSetError::Address => write!(f, "Address is not set"),
            NotSetError::Peers => write!(f, "Peers is not set"),
            NotSetError::WireguardFile => write!(f, "Wireguard File is not set"),
        }
    }
}

impl Error for NotSetError {}

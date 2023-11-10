use anyhow::{Context, Result};
use core::fmt::Formatter;
use reqwest;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt::{self, Display};
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    device: DeviceConfig,
}

impl Config {
    pub fn get_device(&self) -> &DeviceConfig {
        &self.device
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DeviceConfig {
    uuid: String,
    fleet_uuid: String,
    interface: String,
    api_url: String,
    file: PathBuf,
}

impl DeviceConfig {
    pub fn default() -> DeviceConfig {
        DeviceConfig {
            uuid: String::from(""),
            fleet_uuid: String::from(""),
            interface: String::from(""),
            api_url: String::from("https://api.roboticsdeployment.com/device"),
            file: PathBuf::from("/etc/rd/cfg.yaml"),
        }
    }

    pub fn get_uuid(&self) -> &str {
        &self.uuid
    }

    pub fn get_interface(&self) -> &str {
        &self.interface
    }

    pub fn get_fleet_uuid(&self) -> &str {
        &self.fleet_uuid
    }

    pub fn get_api_url(&self) -> &str {
        &self.api_url
    }

    pub fn get_file(&self) -> &PathBuf {
        &self.file
    }

    pub fn set_uuid(&mut self, uuid: String) -> &Self {
        self.uuid = uuid;
        self
    }

    pub fn set_interface(&mut self, interface: String) -> &Self {
        self.interface = interface;
        self
    }

    pub fn set_fleet_uuid(&mut self, fleet: String) -> &Self {
        self.fleet_uuid = fleet;
        self
    }

    pub fn set_api_url(&mut self, api_url: String) -> &Self {
        self.api_url = api_url;
        self
    }

    pub fn set_file(&mut self, file: PathBuf) -> Self {
        self.file = file;
        self.clone()
    }

    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.uuid == "" {
            Err(ValidationError::UuidNotSet)?;
        } else if self.fleet_uuid == "" {
            Err(ValidationError::FleetNotSet)?;
        } else if self.interface == "" {
            Err(ValidationError::InterfaceNotSet)?;
        } else if self.api_url == "" {
            Err(ValidationError::ApiUrlNotSet)?;
        } else if self.file == PathBuf::from("") {
            Err(ValidationError::FileNotSet)?;
        }
        Ok(())
    }

    pub async fn fetch(&self) -> Result<DeviceConfig> {
        let response = reqwest::Client::new()
            .post(self.get_api_url())
            .json(&self)
            .send()
            .await?;
        let device_config: DeviceConfig = response.json().await?;
        Ok(device_config)
    }
}
impl fmt::Display for DeviceConfig {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "uuid: {}, fleet: {}, interface: {}, api_url: {}",
            self.uuid, self.fleet_uuid, self.interface, self.api_url
        )
    }
}

#[derive(Debug)]
pub enum ValidationError {
    UuidNotSet,
    FleetNotSet,
    InterfaceNotSet,
    ApiUrlNotSet,
    FileNotSet,
}

// Implement Display for your custom errors to provide a user-friendly description
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

// Implement the Error trait for your custom error type
impl Error for ValidationError {}

pub fn get_config(rd_file: &PathBuf) -> Result<DeviceConfig> {
    let mut rd_conf: File = File::open(&rd_file)
        .with_context(|| format!("Unable to open rd config file at {:?}", rd_file))?;
    let mut rd_contents = String::new();
    rd_conf
        .read_to_string(&mut rd_contents)
        .context("Unable to read rd config file")?;
    let config: Config =
        serde_yaml::from_str(&rd_contents).context("Unable to deserialize rd YAML")?;
    let device_config = config.get_device().clone();

    Ok(device_config)
}

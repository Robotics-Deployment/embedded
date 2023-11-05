use anyhow::{Context, Result};
use core::fmt::Formatter;
use serde::{Deserialize, Serialize};
use std::fmt;
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
    fleet: String,
    interface: String,
    api_url: String,
    file: PathBuf,
}

impl DeviceConfig {
    pub fn default() -> DeviceConfig {
        DeviceConfig {
            uuid: String::from(""),
            fleet: String::from(""),
            interface: String::from(""),
            api_url: String::from(""),
            file: PathBuf::from("/etc/rd/cfg.yaml"),
        }
    }

    pub fn get_uuid(&self) -> &str {
        &self.uuid
    }

    pub fn get_interface(&self) -> &str {
        &self.interface
    }

    pub fn get_fleet(&self) -> &str {
        &self.fleet
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

    pub fn set_fleet(&mut self, fleet: String) -> &Self {
        self.fleet = fleet;
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

    pub fn validate(&self) -> Result<()> {
        if self.uuid == "" {
            Err(anyhow::anyhow!("UUID is not set"))?;
        }
        if self.fleet == "" {
            Err(anyhow::anyhow!("Fleet is not set"))?;
        }
        if self.interface == "" {
            Err(anyhow::anyhow!("Interface is not set"))?;
        }
        if self.api_url == "" {
            Err(anyhow::anyhow!("API URL is not set"))?;
        }
        if self.file == PathBuf::from("") {
            Err(anyhow::anyhow!("File is not set"))?;
        }
        Ok(())
    }
}
impl fmt::Display for DeviceConfig {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "uuid: {}, fleet: {}, interface: {}, api_url: {}",
            self.uuid, self.fleet, self.interface, self.api_url
        )
    }
}

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

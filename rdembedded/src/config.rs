use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use anyhow::{Result, Context};

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
    interface: String,
}

impl DeviceConfig {
    pub fn get_uuid(&self) -> &str {
        &self.uuid
    }

    pub fn get_interface(&self) -> &str {
        &self.interface
    }
}


pub fn get_config(rd_file: &PathBuf) -> Result<DeviceConfig> {
    let mut rd_conf: File = File::open(&rd_file)
        .with_context(|| format!("Unable to open rd config file at {:?}", rd_file))?;
    let mut rd_contents = String::new();
    rd_conf
        .read_to_string(&mut rd_contents)
        .context("Unable to read rd config file")?;
    let config: Config = serde_yaml::from_str(&rd_contents)
        .context("Unable to deserialize rd YAML")?;
    let device_config = config.get_device().clone();

    Ok(device_config)
}

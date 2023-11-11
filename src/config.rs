use anyhow::{Context, Result};
use core::fmt::Formatter;
use reqwest;
use serde::{Deserialize, Serialize};
use std::fmt::{self, Display};
use std::fs::File;
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom};
use std::path::PathBuf;

use crate::errors::ValidationError;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub device: Device,
    pub wireguard: Wireguard,
}

impl Config {
    pub fn default() -> Config {
        Config {
            device: Device::default(),
            wireguard: Wireguard::default(),
        }
    }

    pub fn get_device(&self) -> &Device {
        &self.device
    }

    pub fn get_wireguard(&self) -> &Wireguard {
        &self.wireguard
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Device {
    uuid: String,
    fleet_uuid: String,
    interface: String,
    api_url: String,
    file: PathBuf,
}

impl Device {
    pub fn default() -> Device {
        Device {
            uuid: String::from(""),
            fleet_uuid: String::from(""),
            interface: String::from("/etc/wireguard/rd0.conf"),
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

    pub fn set_file(&mut self, file: PathBuf) -> &Self {
        self.file = file;
        self
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

    pub async fn fetch(&self) -> Result<Device> {
        let response = reqwest::Client::new()
            .post(self.get_api_url())
            .json(&self)
            .send()
            .await?;
        let device_config: Device = response.json().await?;
        Ok(device_config)
    }

    pub fn load_config(&mut self, rd_file: &PathBuf) -> Result<Device> {
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
}

impl fmt::Display for Device {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "uuid: {}, fleet: {}, interface: {}, api_url: {}",
            self.uuid, self.fleet_uuid, self.interface, self.api_url
        )
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Wireguard {
    pub hub_ip: String,
    pub hub_port: u16,
    pub device_ip: String,
    pub file: PathBuf,
}

impl Wireguard {
    pub fn default() -> Wireguard {
        Wireguard {
            hub_ip: String::from(""),
            hub_port: 0,
            device_ip: String::from(""),
            file: PathBuf::from("/etc/wireguard/rd0.conf"),
        }
    }

    pub fn get_hub_ip(&self) -> &str {
        &self.hub_ip
    }

    pub fn get_hub_port(&self) -> u16 {
        self.hub_port
    }

    pub fn get_device_ip(&self) -> &str {
        &self.device_ip
    }

    pub fn get_file(&self) -> &PathBuf {
        &self.file
    }

    pub fn set_hub_ip(&mut self, hub_ip: String) -> &Self {
        self.hub_ip = hub_ip;
        self
    }

    pub fn set_hub_port(&mut self, hub_port: u16) -> &Self {
        self.hub_port = hub_port;
        self
    }

    pub fn set_device_ip(&mut self, device_ip: String) -> &Self {
        self.device_ip = device_ip;
        self
    }

    pub fn set_file(&mut self, file: PathBuf) -> &Self {
        self.file = file;
        self
    }

    pub fn load_config(&mut self, wg_file: &PathBuf) -> Result<Wireguard> {
        let wg_conf = File::open(&wg_file)
            .with_context(|| format!("Unable to open wireguard config file at {:?}", wg_file))?;
        let mut reader = BufReader::new(wg_conf);

        let hub_ip = Wireguard::scan(&mut reader, "[Peer]", "AllowedIPs")
            .context("Failed to scan for hub IP in wireguard config")?;
        let hub_port = Wireguard::scan(&mut reader, "[Peer]", "Endpoint")
            .context("Failed to scan for hub port in wireguard config")?;
        let device_ip = Wireguard::scan(&mut reader, "[Interface]", "Address")
            .context("Failed to scan for device IP in wireguard config")?;

        let wg_config = Wireguard {
            hub_ip: hub_ip
                .split("/")
                .nth(0)
                .ok_or_else(|| anyhow::anyhow!("Invalid hub IP format"))?
                .to_string(),
            hub_port: hub_port
                .split(":")
                .nth(1)
                .ok_or_else(|| anyhow::anyhow!("Invalid hub port format"))?
                .parse::<u16>()?,
            device_ip: device_ip
                .split("/")
                .nth(0)
                .ok_or_else(|| anyhow::anyhow!("Invalid device IP format"))?
                .to_string(),
            file: wg_file.clone(),
        };

        Ok(wg_config)
    }

    fn scan(reader: &mut BufReader<File>, device: &str, field: &str) -> Result<String> {
        let mut inside_peer_section = false;
        let mut field_value = String::new();
        for line in reader.lines() {
            let line = line.unwrap();
            if line.trim() == device {
                inside_peer_section = true;
            } else if inside_peer_section {
                if line.starts_with(field) {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() > 2 {
                        let ip_address: &str = parts[2];
                        if !ip_address.is_empty() {
                            field_value = ip_address.to_string();
                            break;
                        }
                    }
                }
            }
        }
        // Reset the reader to the beginning of the file
        reader.seek(SeekFrom::Start(0)).unwrap();
        if field_value.is_empty() {
            return Err(anyhow::anyhow!("Unable to find {} in {}", field, device));
        } else {
            return Ok(field_value);
        }
    }
}

impl Display for Wireguard {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "hub_ip: {}, hub_port: {}, device_ip: {}, file: {}",
            self.hub_ip,
            self.hub_port,
            self.device_ip,
            self.file.display()
        )
    }
}

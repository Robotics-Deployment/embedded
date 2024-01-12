use anyhow::{Context, Result};
use async_trait::async_trait;
use core::fmt::Formatter;
use ini::Ini;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::fmt;
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;

use crate::errors::NotSetError;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Device {
    pub created_at: u64,
    pub uuid: String,
    pub fleet_uuid: String,
    pub api_url: String,
    pub file: PathBuf,
    pub wireguard_uuid: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WireGuard {
    pub created_at: u64,
    pub uuid: String,
    pub device_uuid: String,
    pub interface: Interface,
    pub peers: Vec<Peer>,
    pub api_url: String,
    pub file: PathBuf,
    pub wireguard_file: PathBuf,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Interface {
    pub private_key: String,
    pub address: String,
    pub listen_port: Option<u16>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Peer {
    pub public_key: String,
    pub allowed_ips: Vec<String>,
    pub endpoint: Option<String>,
}

#[async_trait]
pub trait Configurable: Serialize + DeserializeOwned {
    fn get_api_url(&self) -> &str;
    fn get_file_path(&self) -> &PathBuf;
    async fn fetch(&self) -> Result<Self>
    where
        Self: Sized,
    {
        let response = reqwest::Client::new()
            .post(self.get_api_url())
            .json(&self)
            .send()
            .await?;
        let config: Self = response.json().await?;
        Ok(config)
    }
    fn load_config(file_path: &PathBuf) -> Result<Self>
    where
        Self: Sized,
    {
        let mut file = File::open(file_path)
            .with_context(|| format!("Unable to open config file at {:?}", file_path))?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .context("Unable to read config file")?;
        let config: Self = serde_yaml::from_str(&contents).context("Unable to deserialize YAML")?;
        Ok(config)
    }
    fn save_config(&self) -> Result<()> {
        let file_path = self.get_file_path();
        let mut file = File::create(file_path)?;
        let yaml = serde_yaml::to_string(&self)?;
        file.write_all(yaml.as_bytes())?;
        Ok(())
    }
}

pub trait Validatable {
    fn validate(&self) -> Result<&Self, NotSetError>;
}

impl Configurable for Device {
    fn get_api_url(&self) -> &str {
        &self.api_url
    }
    fn get_file_path(&self) -> &PathBuf {
        &self.file
    }
}

impl Configurable for WireGuard {
    fn get_api_url(&self) -> &str {
        &self.api_url
    }
    fn get_file_path(&self) -> &PathBuf {
        &self.file
    }
}

impl Validatable for Device {
    fn validate(&self) -> Result<&Self, NotSetError> {
        if self.created_at == 0 {
            Err(NotSetError::CreatedAt)?;
        } else if self.uuid.is_empty() {
            Err(NotSetError::Uuid)?;
        } else if self.fleet_uuid.is_empty() {
            Err(NotSetError::Fleet)?;
        } else if self.api_url.is_empty() {
            Err(NotSetError::ApiUrl)?;
        } else if self.file == PathBuf::from("") {
            Err(NotSetError::File)?;
        }
        Ok(self)
    }
}

impl Validatable for WireGuard {
    fn validate(&self) -> Result<&Self, NotSetError> {
        if self.created_at == 0 {
            Err(NotSetError::CreatedAt)?;
        } else if self.interface.private_key.is_empty() {
            Err(NotSetError::PrivateKey)?;
        } else if self.interface.address.is_empty() {
            Err(NotSetError::Address)?;
        } else if self.peers.is_empty() {
            Err(NotSetError::Peers)?;
        } else if self.api_url.is_empty() {
            Err(NotSetError::ApiUrl)?;
        } else if self.file == PathBuf::from("") {
            Err(NotSetError::File)?;
        } else if self.wireguard_file == PathBuf::from("") {
            Err(NotSetError::WireguardFile)?;
        }
        Ok(self)
    }
}

impl WireGuard {
    pub fn load_from_wireguard_file(self, path: &PathBuf) -> Result<WireGuard> {
        let conf = Ini::load_from_file(path)?;
        let mut interface = Interface {
            private_key: String::new(),
            address: String::new(),
            listen_port: None,
        };
        let mut peers = Vec::new();
        for (sec, prop) in conf.iter() {
            match sec {
                Some(section) if section.starts_with("Interface") => {
                    for (key, value) in prop.iter() {
                        match key {
                            "PrivateKey" => interface.private_key = value.into(),
                            "Address" => interface.address = value.into(),
                            "ListenPort" => interface.listen_port = value.parse().ok(),
                            // Handle other fields
                            _ => {}
                        }
                    }
                }
                Some(section) if section.starts_with("Peer") => {
                    let mut peer = Peer {
                        public_key: String::new(),
                        allowed_ips: Vec::new(),
                        endpoint: None,
                    };
                    for (key, value) in prop.iter() {
                        match key {
                            "PublicKey" => peer.public_key = value.into(),
                            "AllowedIPs" => {
                                peer.allowed_ips = value.split(',').map(String::from).collect()
                            }
                            "Endpoint" => peer.endpoint = Some(value.into()),
                            // Handle other fields
                            _ => {}
                        }
                    }
                    peers.push(peer);
                }
                _ => {}
            }
        }
        Ok(WireGuard {
            interface,
            peers,
            ..self
        })
    }
}

impl WireGuard {
    pub fn new(created_at: u64, wireguard_uuid: String, device_uuid: String) -> WireGuard {
        WireGuard {
            created_at,
            device_uuid,
            uuid: wireguard_uuid,
            interface: Interface {
                private_key: String::new(),
                address: String::new(),
                listen_port: None,
            },
            peers: Vec::new(),
            api_url: "https://api.roboticsdeployment.com/wireguard".into(),
            file: "/etc/rd/wireguard.yaml".into(),
            wireguard_file: "/etc/wireguard/rd0.conf".into(),
        }
    }
}

impl Default for WireGuard {
    fn default() -> Self {
        Self::new(0, String::new(), String::new())
    }
}

impl fmt::Display for Device {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "uuid: {}, fleet: {}, api_url: {}",
            self.uuid, self.fleet_uuid, self.api_url
        )
    }
}

impl fmt::Display for WireGuard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.interface)?;
        for peer in &self.peers {
            writeln!(f)?;
            write!(f, "{}", peer)?;
        }
        Ok(())
    }
}

impl fmt::Display for Interface {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "[Interface]")?;
        writeln!(f, "PrivateKey = {}", self.private_key)?;
        writeln!(f, "Address = {}", self.address)?;
        if let Some(port) = self.listen_port {
            writeln!(f, "ListenPort = {}", port)?;
        }
        Ok(())
    }
}

impl fmt::Display for Peer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "[Peer]")?;
        writeln!(f, "PublicKey = {}", self.public_key)?;
        writeln!(f, "AllowedIPs = {}", self.allowed_ips.join(", "))?;
        if let Some(endpoint) = &self.endpoint {
            writeln!(f, "Endpoint = {}", endpoint)?;
        }
        Ok(())
    }
}

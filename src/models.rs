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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DeviceConfig {
    pub created_at: u64,
    pub uuid: String,
    pub fleet_uuid: String,
    pub api_url: String,
    pub file: PathBuf,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WireGuardConfig {
    created_at: u64,
    interface: InterfaceConfig,
    peers: Vec<PeerConfig>,
    api_url: String,
    file: PathBuf,
    wg_file: PathBuf,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InterfaceConfig {
    private_key: String,
    address: String,
    listen_port: Option<u16>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PeerConfig {
    public_key: String,
    allowed_ips: Vec<String>,
    endpoint: Option<String>,
}

impl Configurable for DeviceConfig {
    fn get_api_url(&self) -> &str {
        &self.api_url
    }
    fn get_file_path(&self) -> &PathBuf {
        &self.file
    }
}

impl Validatable for DeviceConfig {
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

impl Validatable for WireGuardConfig {
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
        } else if self.wg_file == PathBuf::from("") {
            Err(NotSetError::WireguardFile)?;
        }
        Ok(self)
    }
}

impl WireGuardConfig {
    pub fn load_from_wireguard_config(self, path: &PathBuf) -> Result<WireGuardConfig> {
        let conf = Ini::load_from_file(path)?;
        let mut interface = InterfaceConfig {
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
                    let mut peer = PeerConfig {
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
        Ok(WireGuardConfig {
            interface,
            peers,
            ..self
        })
    }

    pub fn save_to_wireguard_config(config: &WireGuardConfig, path: &PathBuf) -> Result<()> {
        let mut conf = Ini::new();
        {
            let interface_section = conf.section_mut(Some("Interface".to_owned())).unwrap();
            interface_section.insert(
                "PrivateKey".to_owned(),
                config.interface.private_key.clone(),
            );
            interface_section.insert("Address".to_owned(), config.interface.address.clone());
            if let Some(port) = config.interface.listen_port {
                interface_section.insert("ListenPort".to_owned(), port.to_string());
            }
        }
        for peer in &config.peers {
            let peer_section = conf.section_mut(Some("Peer".to_owned())).unwrap();
            peer_section.insert("PublicKey".to_owned(), peer.public_key.clone());
            peer_section.insert("AllowedIPs".to_owned(), peer.allowed_ips.join(","));
            if let Some(endpoint) = &peer.endpoint {
                peer_section.insert("Endpoint".to_owned(), endpoint.clone());
            }
        }
        conf.write_to_file(path).unwrap();
        Ok(())
    }
}

impl fmt::Display for DeviceConfig {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "uuid: {}, fleet: {}, api_url: {}",
            self.uuid, self.fleet_uuid, self.api_url
        )
    }
}

impl fmt::Display for WireGuardConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.interface)?;
        for peer in &self.peers {
            writeln!(f)?;
            write!(f, "{}", peer)?;
        }
        Ok(())
    }
}

impl fmt::Display for InterfaceConfig {
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

impl fmt::Display for PeerConfig {
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

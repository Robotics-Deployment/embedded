use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufRead, BufReader, Seek, SeekFrom};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct WgConfig {
    hub_ip: String,
    hub_port: u16,
    device_ip: String,
    file: PathBuf,
}

impl WgConfig {
    pub fn default() -> WgConfig {
        WgConfig {
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
}

pub fn get_config(wg_file: &PathBuf) -> Result<WgConfig> {
    let wg_conf = File::open(&wg_file)
        .with_context(|| format!("Unable to open wireguard config file at {:?}", wg_file))?;
    let mut reader = BufReader::new(wg_conf);

    let hub_ip = scan(&mut reader, "[Peer]", "AllowedIPs")
        .context("Failed to scan for hub IP in wireguard config")?;
    let hub_port = scan(&mut reader, "[Peer]", "Endpoint")
        .context("Failed to scan for hub port in wireguard config")?;
    let device_ip = scan(&mut reader, "[Interface]", "Address")
        .context("Failed to scan for device IP in wireguard config")?;

    let wg_config = WgConfig {
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

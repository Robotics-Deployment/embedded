use ini::Ini;
use ini::{Properties, SectionEntry};
use log::{error, info};
use std::path::PathBuf;

use crate::errors;
use crate::models::{self, Configurable, Validatable, WireGuard};

pub async fn initialize_device_config() -> Result<models::DeviceConfig, String> {
    // Load the device configuration from a file
    let config_path = PathBuf::from("/etc/rd/device.yaml");
    let device = match models::DeviceConfig::load_config(&config_path) {
        Ok(cfg) => {
            info!("Using device config file: {:?}", cfg.file);
            cfg
        }
        Err(e) => {
            return Err(format!("Unable to read configuration file: {}", e));
        }
    };
    // Validate the loaded configuration
    match device.validate() {
        Ok(_) => {
            info!("Device config validated: {:?}", device);
            Ok(device)
        }
        Err(errors::NotSetError::Fleet) => {
            let msg = "Fleet not set in configuration file. Device does not know which fleet it belongs to. Cannot continue...";
            Err(msg.to_string())
        }
        Err(errors::NotSetError::Uuid) => {
            info!("Device UUID not set in configuration file, fetching...");
            match device.fetch().await {
                Ok(fetched_device) => {
                    info!("Successfully fetched configuration");
                    Ok(fetched_device)
                }
                Err(error) => Err(format!("Unable to fetch configuration: {}", error)),
            }
        }
        Err(e) => Err(format!(
            "Unhandled error validating configuration file: {}",
            e
        )),
    }
}

pub async fn initialize_wireguard_config(
    device: &models::DeviceConfig,
) -> Result<models::WireGuard, String> {
    let config_path = PathBuf::from("/etc/rd/wireguard.yaml");
    match models::WireGuard::load_config(&config_path) {
        Ok(cfg) => {
            info!("WireGuard config loaded successfully.");
            Ok(cfg)
        }
        Err(e) => {
            error!("Unable to read WireGuard configuration file: {}", e);
            let wireguard = models::WireGuard::new(
                device.created_at,
                device.wireguard_uuid.clone(),
                device.uuid.clone(),
            );
            info!("Fetching new WireGuard configuration...");
            match wireguard.fetch().await {
                Ok(wg) => {
                    info!("Successfully fetched new WireGuard configuration.");
                    match wg.save_config() {
                        Ok(_) => {
                            info!("Successfully saved new WireGuard configuration.");
                            Ok(wg)
                        }
                        Err(e) => Err(format!("Unable to save new WireGuard configuration: {}", e)),
                    }
                }
                Err(error) => Err(format!(
                    "Unable to fetch new WireGuard configuration: {}",
                    error
                )),
            }
        }
    }
}

pub fn save_to_wireguard_file(wireguard: &WireGuard) -> Result<(), String> {
    let mut conf = Ini::new();

    // Interface section
    conf.with_section(Some("Interface"))
        .set("PrivateKey", &wireguard.interface.private_key)
        .set("Address", &wireguard.interface.address);
    if let Some(port) = wireguard.interface.listen_port {
        conf.with_section(Some("Interface"))
            .set("ListenPort", &port.to_string());
    }

    // Peer sections
    for (_, peer) in wireguard.peers.iter().enumerate() {
        let section_name = "Peer".to_string();
        match conf.entry(Some(section_name)) {
            SectionEntry::Vacant(vac) => {
                let mut entry = Properties::new();
                entry.insert("PublicKey", &peer.public_key);
                entry.insert("AllowedIPs", &peer.allowed_ips.join(","));
                if let Some(endpoint) = &peer.endpoint {
                    entry.insert("Endpoint", endpoint);
                }
                vac.insert(entry);
            }

            SectionEntry::Occupied(mut occ) => {
                let mut entry = Properties::new();
                entry.insert("PublicKey", &peer.public_key);
                entry.insert("AllowedIPs", &peer.allowed_ips.join(","));
                if let Some(endpoint) = &peer.endpoint {
                    entry.insert("Endpoint", endpoint);
                }
                occ.append(entry);
            }
        }
    }

    // Write the configuration to the file
    conf.write_to_file(wireguard.wireguard_file.clone())
        .map_err(|e| format!("Unable to write WireGuard configuration: {}", e))
}

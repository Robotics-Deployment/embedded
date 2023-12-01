use env_logger::Builder;
use log::{error, info, LevelFilter};
use nix::unistd::getuid;
use std::net::ToSocketAddrs;
use std::path::PathBuf;
use std::process::{exit, Command};
use std::str::FromStr;
use tokio::net::{unix::SocketAddr, UdpSocket};
use tokio::time::{interval, Duration};

use crate::models::{Configurable, Validatable};

mod errors;
mod models;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    Builder::new().filter_level(LevelFilter::Info).init();
    info!("Starting Rdembedded");

    let mut device: models::DeviceConfig;
    let mut wireguard: models::WireGuard;

    // Memory Scope
    {
        if !getuid().is_root() {
            // Due to wireguard configurations requiring root access
            error!("This program must be run as root.");
            exit(1);
        }

        // Device
        let r = models::DeviceConfig::load_config(&PathBuf::from("/etc/rd/device.yaml"));

        device = match r {
            Err(e) => {
                error!("Unable to read configuration file: {}", e);
                exit(1);
            }
            Ok(cfg) => {
                info!("Using device config file: {:?}", cfg.file);
                cfg
            }
        };

        let r = device.validate();

        device = match r {
            Err(e) => match e {
                errors::NotSetError::Fleet => {
                    error!("Fleet not set in configuration file. Device does not know which fleet it belongs to. cannot continue...");
                    exit(1);
                }
                errors::NotSetError::Uuid => {
                    info!("Device UUID not set in configuration file, fetching...");
                    let result_fetch = device.fetch().await;
                    match result_fetch {
                        Err(error) => {
                            error!("Unable to fetch configuration: {}", error);
                            exit(1);
                        }
                        Ok(dev) => {
                            info!("Successfully fetched configuration");
                            dev
                        }
                    }
                }
                _ => {
                    error!("Unhandled error validating configuration file: {}", e);
                    exit(1);
                }
            },
            Ok(dev) => {
                info!("device config validated: {:?}", device);
                dev.clone()
            }
        };

        // Wireguard
        let r = models::WireGuard::load_config(&PathBuf::from("/etc/rd/wireguard.yaml"));
        wireguard = match r {
            Err(e) => {
                error!("Unable to read configuration file: {}", e);
                wireguard = models::WireGuard::new(device.created_at, device.uuid.clone());

                match wireguard.fetch().await {
                    Err(error) => {
                        error!("Unable to fetch configuration: {}", error);
                        exit(1);
                    }
                    Ok(wg) => {
                        info!("Successfully fetched configuration");
                        match wg.save_config() {
                            Ok(_) => {
                                info!("Successfully saved configuration");
                            }
                            Err(e) => {
                                error!("Unable to save configuration: {}", e);
                                exit(1);
                            }
                        }
                        wg
                    }
                }
            }
            Ok(cfg) => {
                info!("Using wireguard config file: {:?}", cfg.file);
                cfg
            }
        };

        wireguard = match wireguard
            .save_to_wireguard_file(PathBuf::from_str("/etc/wireguard/rd0.conf").unwrap())
        {
            Err(e) => {
                error!("Unable to save wireguard configuration: {}", e);
                exit(1);
            }
            Ok(wg) => {
                info!("Successfully saved wireguard configuration");
                wg
            }
        };

        let output = Command::new("wg-quick").args(["up", "rd0"]).output()?;
        if !output.status.success() {
            error!(
                "Failed to bring up WireGuard interface: {}",
                String::from_utf8_lossy(&output.stderr)
            );
            std::process::exit(1);
        }
        println!("WireGuard interface is up.");
    }

    let source_address = match wireguard.interface.address.to_socket_addrs() {
        Ok(mut addrs) => addrs.next().ok_or_else(|| {
            error!("Failed to parse interface address");
            exit(1);
        }),
        Err(e) => {
            error!("Failed to resolve address: {}", e);
            exit(1);
        }
    };

    let destination_address = match wireguard.peers.get(0) {
        Some(peer) => match &peer.endpoint {
            Some(endpoint) => match endpoint.to_socket_addrs() {
                Ok(mut addrs) => addrs.next().ok_or_else(|| {
                    std::io::Error::new(std::io::ErrorKind::NotFound, "No address found")
                })?,
                Err(e) => {
                    error!("Failed to resolve endpoint: {}", e);
                    exit(1);
                }
            },
            None => {
                error!("No endpoint found");
                exit(1);
            }
        },
        None => {
            error!("No peer found");
            exit(1);
        }
    };

    let socket = match UdpSocket::bind(format!("[{}]:{}", source_address.unwrap(), 42069)).await {
        Ok(s) => s,
        Err(e) => {
            error!("Failed to bind socket: {}", e);
            exit(1);
        }
    };
    let mut interval = interval(Duration::from_secs(1));

    loop {
        socket
            .send_to(&device.uuid.clone().into_bytes(), &destination_address)
            .await?;
        interval.tick().await;
    }
}

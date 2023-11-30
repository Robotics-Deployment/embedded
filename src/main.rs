use env_logger::Builder;
use log::{error, info, warn, LevelFilter};
use nix::unistd::getuid;
use std::path::PathBuf;
use std::process::exit;
use tokio::net::UdpSocket;
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

    let socket: Option<UdpSocket>;
    let mut interval = interval(Duration::from_secs(1));

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
    }
    // TODO: Continue with wireguard configuration
    //
    // Wireguard
    let r = models::WireGuard::load_config(&PathBuf::from("/etc/rd/wireguard.yaml"));
    //TODO: Instantiation makes no sense when you want to fetch.
    wireguard = match r {
        Err(e) => {
            error!("Unable to read configuration file: {}", e);
            wireguard = models::WireGuard::new();
            }
            match models::WireGuard::fetch().await {
                Err(error) => {
                    error!("Unable to fetch configuration: {}", error);
                    exit(1);
                }
                Ok(wg) => {
                    info!("Successfully fetched configuration");
                    wg
                }
            }
        }
        Ok(cfg) => {
            info!("Using wireguard config file: {:?}", cfg.file);
            cfg
        }
    };

    Ok(())
}

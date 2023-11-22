use env_logger::Builder;
use log::{error, info, warn, LevelFilter};
use nix::unistd::getuid;
use std::path::PathBuf;
use std::process::exit;
use tokio::net::UdpSocket;
use tokio::time::{interval, Duration};
use uuid::Uuid;

mod errors;
mod models;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    Builder::new().filter_level(LevelFilter::Info).init();
    info!("Starting rdembedded");

    let mut device: models::Device;
    let mut wireguard: models::Wireguard;

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
        let r = models::Device::load_config(&PathBuf::from("/etc/rd/cfg.yaml"));

        device = match r {
            Err(e) => {
                error!("Unable to read configuration file: {}", e);
                exit(1);
            }
            Ok(cfg) => {
                info!("Using device config file: {:?}", cfg.get_file());
                cfg
            }
        };

        let r = device.validate();

        device = match r {
            Err(e) => match e {
                errors::ValidationNotSetError::Fleet => {
                    error!("Fleet not set in configuration file. Device does not know which fleet it belongs to. cannot continue...");
                    exit(1);
                }
                errors::ValidationNotSetError::Uuid => {
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
        let r = models::Wireguard::load_config(&PathBuf::from("/etc/wireguard/rd0.conf"));

        wireguard = match r {
            Err(e) => {
                error!("Unable to read wireguard config file: {}", e);
                let result_fetch = models::Wireguard::fetch(&models::Wireguard::init()).await;
                match result_fetch {
                    Err(error) => {
                        error!("Unable to fetch wireguard configuration: {}", error);
                        exit(1);
                    }
                    Ok(cfg) => {
                        info!("Successfully fetched wireguard configuration");
                        cfg
                    }
                }
            }
            Ok(cfg) => {
                info!("Using wireguard config file {:?}", cfg.get_file());
                cfg
            }
        };

        wireguard = wireguard.set_uuid(device.get_uuid().to_string());
        info!("wireguard config: {:?}", wireguard);

        // Socket Setup
        println!(
            "Creating UDP socket {}:{}",
            wireguard.get_device_ip(),
            wireguard.get_hub_port()
        );

        loop {
            let r = UdpSocket::bind(format!(
                "{}:{}",
                wireguard.get_device_ip(),
                wireguard.get_hub_port()
            ))
            .await;
            if r.is_ok() {
                socket = Some(r.unwrap());
                println!(
                    "Bound socket to {}:{}",
                    wireguard.get_device_ip(),
                    wireguard.get_hub_port()
                );
                break;
            }
            let err_str = r.err().unwrap().to_string();
            error!(
                "{}",
                format!("Unable to bind socket, retrying in 1 second: {err_str}")
            );
            interval.tick().await;
        }

        info!(
            "Sending heartbeat packet to {}:{}",
            wireguard.get_hub_ip(),
            wireguard.get_hub_port()
        );
    }

    let uuid_reader = Uuid::parse_str(device.get_uuid());
    let uuid = match uuid_reader {
        Err(e) => {
            error!("Unable to parse UUID: {}", e);
            exit(1);
        }
        Ok(uuid) => uuid,
    };

    // Main
    let mut once = true;
    loop {
        let r = socket
            .as_ref()
            .unwrap()
            .send_to(
                uuid.as_bytes(),
                format!("{}:{}", wireguard.get_hub_ip(), wireguard.get_hub_port()),
            )
            .await;

        if r.is_err() {
            warn!("Unable to send heartbeat packet, retrying in 1 second");
            once = true;
            interval.tick().await;
            continue;
        }

        if once {
            info!(
                "Successfully sent heartbeat packet with UUID: {}",
                uuid.to_string()
            );
            info!("Sending heartbeat packet every second...");
            once = false;
        }

        interval.tick().await;
    }
}

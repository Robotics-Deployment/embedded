use anyhow;
use env_logger::Builder;
use log::{error, info, warn, LevelFilter};
use nix;
use nix::unistd::getuid;
use std::process::exit;
use tokio::net::UdpSocket;
use tokio::time::{interval, Duration};
use uuid::Uuid;

mod config;
mod errors;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    Builder::new().filter_level(LevelFilter::Info).init();
    info!("Starting rdembedded");

    let socket: Option<UdpSocket>;
    let mut config = config::Config::default();
    let mut interval = interval(Duration::from_secs(1));

    // Memory Scope
    {
        if !getuid().is_root() {
            // Due to wireguard configurations requiring root access
            error!("This program must be run as root.");
            exit(1);
        }

        // Device
        let mut device: config::Device = config.get_device().clone();
        let r = device.load_config(device.clone().get_file());

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

        match r {
            Err(e) => match e {
                errors::ValidationError::FleetNotSet => {
                    error!("Fleet not set in configuration file. Device does not know which fleet it belongs to. cannot continue...");
                    exit(1);
                }
                errors::ValidationError::UuidNotSet => {
                    info!("Device UUID not set in configuration file, fetching...");
                    let result_fetch = device.fetch().await;
                    match result_fetch {
                        Err(error) => {
                            error!("Unable to fetch configuration: {}", error);
                            exit(1);
                        }
                        Ok(fetched) => {
                            info!("Successfully fetched configuration");
                            device = fetched;
                        }
                    }
                }
                _ => {
                    error!("Unhandled error validating configuration file: {}", e);
                    exit(1);
                }
            },
            Ok(_) => {
                info!("device config validated: {:?}", device);
            }
        }
        config.device = device;

        // Wireguard
        let mut wireguard: config::Wireguard = config.get_wireguard().clone();
        let r = wireguard.load_config(wireguard.clone().get_file());

        match r {
            Err(e) => {
                error!("Unable to read wireguard config file: {}", e);
                exit(1);
            }
            Ok(cfg) => {
                wireguard = cfg;
                info!("Using wireguard config file {:?}", wireguard.get_file());
            }
        }

        info!("wireguard config validated: {:?}", wireguard);

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
            println!(
                "{}",
                format!("Unable to bind socket, retrying in 1 second: {err_str}")
            );
            interval.tick().await;
        }

        println!(
            "Sending heartbeat packet to {}:{}",
            wireguard.get_hub_ip(),
            wireguard.get_hub_port()
        );
        config.wireguard = wireguard;
    }

    let uuid_reader = Uuid::parse_str(config.get_device().get_uuid());
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
                format!(
                    "{}:{}",
                    config.get_wireguard().get_hub_ip(),
                    config.get_wireguard().get_hub_port()
                ),
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

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
mod wg;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    Builder::new().filter_level(LevelFilter::Info).init();
    info!("Starting rdembedded");

    let socket: Option<UdpSocket>;
    let mut wg_config = wg::WgConfig::default();
    let mut device_config = config::DeviceConfig::default();
    let mut interval = interval(Duration::from_secs(1));

    // Memory Scope
    {
        if !getuid().is_root() {
            // Due to wireguard configurations requiring root access
            error!("This program must be run as root.");
            exit(1);
        }

        let r = config::get_config(&device_config.get_file());
        if r.is_err() {
            error!("Unable to read configuration file");
            exit(1);
        }

        device_config = r.unwrap();
        let r = device_config.validate();

        match r {
            Err(e) => match e {
                config::ValidationError::UuidNotSet => {
                    error!("UUID not set in configuration file, cannot continue...");
                    exit(1);
                }
                config::ValidationError::FleetNotSet => {
                    info!("Fleet not set in configuration file, fetching...");
                    let result_fetch = device_config.fetch().await;
                    match result_fetch {
                        Err(error) => {
                            error!("Unable to fetch configuration: {}", error);
                            exit(1);
                        }
                        Ok(fetched) => {
                            info!("Succeissfully fetched configuration");
                            device_config = fetched;
                        }
                    }
                }
                _ => {
                    error!("Unhandled error validating configuration file: {}", e);
                    exit(1);
                }
            },
            Ok(_) => {
                info!(
                    "Using configuration file {}",
                    device_config.get_file().to_str().unwrap()
                );
            }
        }

        let r = wg::get_config(&wg_config.get_file());

        match r {
            Err(e) => {
                println!("Unable to read wireguard config file: {}", e);
                exit(1);
            }
            Ok(cfg) => {
                wg_config = cfg;
                println!(
                    "Using wireguard config file {}",
                    wg_config.get_file().to_str().unwrap()
                );
            }
        }

        // Status Output
        println!(
            "Using config file: {}",
            device_config.get_file().to_str().unwrap()
        );
        println!("Using device UUID: {}", device_config.get_uuid());
        println!("Using network interface: {}", device_config.get_interface());
        println!("Using fleet UUID: {}", device_config.get_fleet_uuid());
        println!("Using API URL: {}", device_config.get_api_url());
        println!(
            "Using Wireguard config file: {}",
            wg_config.get_file().to_str().unwrap()
        );
        println!("Using Device IP: {}", wg_config.get_device_ip());
        println!("Using Hub IP: {}", wg_config.get_hub_ip());
        println!("Using Hub Port: {}", wg_config.get_hub_port());
        // Socket Setup
        println!(
            "Creating UDP socket {}:{}",
            wg_config.get_device_ip(),
            wg_config.get_hub_port()
        );
        loop {
            let r = UdpSocket::bind(format!(
                "{}:{}",
                wg_config.get_device_ip(),
                wg_config.get_hub_port()
            ))
            .await;
            if r.is_ok() {
                socket = Some(r.unwrap());
                println!(
                    "Bound socket to {}:{}",
                    wg_config.get_device_ip(),
                    wg_config.get_hub_port()
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
            wg_config.get_hub_ip(),
            wg_config.get_hub_port()
        );
    }
    // Main
    let mut once = true;
    let uuid: Uuid = Uuid::parse_str(device_config.get_uuid()).expect("Unable to parse UUID");

    loop {
        let r = socket
            .as_ref()
            .unwrap()
            .send_to(
                uuid.as_bytes(),
                format!("{}:{}", wg_config.get_hub_ip(), wg_config.get_hub_port()),
            )
            .await;

        if r.is_err() {
            println!("Unable to send heartbeat packet, retrying in 1 second");
            interval.tick().await;
            continue;
        }

        if once {
            println!(
                "Successfully sent heartbeat packet with UUID: {}",
                uuid.to_string()
            );
            println!("Sending heartbeat packet every second...");
            once = false;
        }
        interval.tick().await;
    }
}

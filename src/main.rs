extern crate anyhow;
extern crate nix;
extern crate serde;
extern crate serde_yaml;
use nix::unistd::getuid;
use reqwest;
use std::path::PathBuf;
use std::process::exit;
use tokio::net::UdpSocket;
use tokio::time::{interval, Duration};
use uuid::Uuid;

mod config;
mod wg;

async fn fetch_config(cfg: &config::DeviceConfig) -> anyhow::Result<()> {
    let response = reqwest::get(format!("{}/{}", cfg.get_api_url(), cfg.get_uuid())).await?;

    if response.status().is_success() {
        let body = response.text().await?;
        let rd_file = PathBuf::from("/etc/rd/cfg.yaml");
        println!("Writing configuration to {}", rd_file.to_str().unwrap());
        std::fs::write(rd_file, body)?;
    } else {
        Err(anyhow::anyhow!(
            "Unable to fetch configuration from API server"
        ))?;
    }
    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let socket: Option<UdpSocket>;
    let mut wg_config = wg::WgConfig::default();
    let mut device_config = config::DeviceConfig::default();
    let mut interval = interval(Duration::from_secs(1));

    // Memory Scope
    {
        if !getuid().is_root() {
            // Due to wireguard requiring root access
            println!("This program must be run as root.");
            exit(1);
        }

        // Configuration
        let rd_file = PathBuf::from(device_config.get_file());
        let r = config::get_config(&rd_file);

        match r {
            Err(e) => {
                println!("Unable to read config file: {}", e);
                exit(1);
            }
            Ok(cfg) => {
                device_config = cfg;
                println!("Using config file {}", rd_file.to_str().unwrap());
                match device_config.validate() {
                    Err(e) => {
                        println!("Incomplete config file: {}", e);
                        println!("Attempting to fetch from API server");
                        let _ = fetch_config(&device_config).await;
                        exit(1);
                    }
                    Ok(_) => {
                        println!("Configuration is valid");
                    }
                }
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
        println!("Using UUID: {}", device_config.get_uuid());
        println!("Using network interface: {}", device_config.get_interface());
        println!("Using fleet: {}", device_config.get_fleet());
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

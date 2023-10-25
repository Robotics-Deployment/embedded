extern crate serde;
extern crate serde_yaml;
extern crate anyhow;
extern crate nix;

use std::process::exit;
use std::path::PathBuf;
use tokio::net::UdpSocket;
use tokio::time::{Duration, interval};
use uuid::Uuid;
use nix::unistd::getuid;

mod wg;
mod config;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let wg_config: wg::WgConfig;
    let socket: Option<UdpSocket>;
    let device_config: config::DeviceConfig;
    let mut interval = interval(Duration::from_secs(1));

    // Memory Scope
    {
        if !getuid().is_root() {
            println!("This program must be run as root.");
            interval.tick().await;
            exit(1);
        }


        // Configuration
        loop {
            let rd_file = PathBuf::from("/etc/rd/cfg.yaml");
            let r = config::get_config(&rd_file);
            if r.is_ok() {
                device_config = r.unwrap();
                println!("Using config file {}", rd_file.to_str().unwrap());
                break;
            }
            println!("Unable to read config file {}, retrying in 1 second", rd_file.to_str().unwrap());
            interval.tick().await;
        }

        loop {
            let mut wg_file = PathBuf::from("/etc/wireguard/");
            wg_file.push(device_config.get_interface());
            wg_file.set_extension("conf");
            let r = wg::get_config(&wg_file);
            if r.is_ok() {
                wg_config = r.unwrap();
                println!("Using wireguard config file {}", wg_file.to_str().unwrap());
                break;
            }
            println!("Unable to read wireguard config file, retrying in 1 second");
            interval.tick().await;
        }

        // Status Output
        println!("Using UUID: {}", device_config.get_uuid());
        println!("Using network interface: {}", device_config.get_interface());
        println!("Using Device IP: {}", wg_config.get_device_ip());
        println!("Using Hub IP: {}", wg_config.get_hub_ip());
        println!("Using Hub Port: {}", wg_config.get_hub_port());

        // Socket Setup
        println!("Creating UDP socket {}:{}", wg_config.get_device_ip(), wg_config.get_hub_port());
        loop {
            let r = UdpSocket::bind(format!("{}:{}", wg_config.get_device_ip(), wg_config.get_hub_port())).await;
            if r.is_ok() {
                socket = Some(r.unwrap());
                println!("Bound socket to {}:{}", wg_config.get_device_ip(), wg_config.get_hub_port());
                break;
            }
            let err_str = r.err().unwrap().to_string();
            println!("{}", format!("Unable to bind socket, retrying in 1 second: {err_str}"));
            interval.tick().await;
        }

        println!("Sending heartbeat packet to {}:{}", wg_config.get_hub_ip(), wg_config.get_hub_port());
    }
    // Main
    let mut once = true;
    let uuid: Uuid = Uuid::parse_str(device_config.get_uuid()).expect("Unable to parse UUID");

    loop {
        let r = socket
            .as_ref()
            .unwrap()
            .send_to(uuid.as_bytes(),
                     format!("{}:{}",
                             wg_config.get_hub_ip(),
                             wg_config.get_hub_port()))
            .await;

        if r.is_err() {
            println!("Unable to send heartbeat packet, retrying in 1 second");
            interval.tick().await;
            continue;
        }

        if once {
            println!("Successfully sent heartbeat packet with UUID: {}", uuid.to_string());
            println!("Sending heartbeat packet every second...");
            once = false;
        }
        interval.tick().await;
    }
}
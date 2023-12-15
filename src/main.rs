use env_logger::Builder;
use log::{error, info, LevelFilter};
use nix::unistd::getuid;
use std::net::ToSocketAddrs;
use std::process::{exit, Command};
use tokio::net::UdpSocket;
use tokio::time::{interval, Duration};

mod errors;
mod functions;
mod models;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    Builder::new().filter_level(LevelFilter::Info).init();
    info!("Starting Rdembedded");

    let device: models::DeviceConfig;
    let wireguard: models::WireGuard;

    // Memory Scope
    {
        if !getuid().is_root() {
            // Due to wireguard configurations requiring root access
            error!("This program must be run as root.");
            exit(1);
        }

        // Device
        device = match functions::initialize_device_config().await {
            Ok(cfg) => cfg,
            Err(e) => {
                error!("Critical error: {}", e);
                exit(1);
            }
        };

        // Wireguard
        wireguard = match functions::initialize_wireguard_config(&device).await {
            Ok(cfg) => cfg,
            Err(e) => {
                error!("Critical error: {}", e);
                exit(1);
            }
        };

        info!("Using wireguard config file: {:?}", wireguard.file);
        if let Err(e) = functions::save_to_wireguard_file(&wireguard) {
            error!("Unable to save wireguard configuration: {}", e);
            exit(1);
        }

        info!("Successfully saved wireguard configuration");

        let output = Command::new("wg-quick").args(["up", "rd0"]).output()?;
        if !output.status.success() {
            error!(
                "Failed to bring up WireGuard interface: {}",
                String::from_utf8_lossy(&output.stderr)
            );
            exit(1);
        }

        println!("WireGuard interface is up.");
    }

    let mut destination_address = match wireguard.peers.get(0) {
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

    let socket = match UdpSocket::bind("0.0.0.0:0").await {
        Ok(s) => s,
        Err(e) => {
            error!("Failed to bind socket: {}", e);
            exit(1);
        }
    };

    destination_address = std::net::SocketAddr::new(destination_address.ip(), 42069);

    let mut interval = interval(Duration::from_secs(1));
    info!("Sending packets to {}", destination_address);

    loop {
        socket
            .send_to(&device.uuid.clone().into_bytes(), &destination_address)
            .await?;
        interval.tick().await;
    }
}

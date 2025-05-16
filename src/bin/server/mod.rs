use args::ServerArgs;
use discv5::{ConfigBuilder, DefaultProtocolId, Discv5, ListenConfig};
use std::error::Error;
use std::net::UdpSocket;
use std::sync::Arc;
use std::time::Duration;
use tokio::{
    select,
    signal::unix::{signal, SignalKind},
};
use tracing::{info, warn};

use crate::key;

pub mod args;
pub mod echo;
pub mod enr;
pub mod events;
pub mod stats;

pub async fn run(args: ServerArgs) -> Result<(), Box<dyn Error>> {
    let key = key::read_secp256k1_key_from_file(&args.secp256k1_key_file)?;
    let enr = enr::build(&args, &key)?;

    let enr_str: &'static str = Box::leak(enr.to_string().into_boxed_str());

    info!("Node Id: {}", enr.node_id());
    if enr.udp4_socket().is_some() {
        info!("Base64 ENR: {}", enr.to_base64());
        info!(
            "ip: {}, udp port:{}",
            enr.ip4().unwrap(),
            enr.udp4().unwrap()
        );
    } else {
        warn!("ENR is not printed as no IP:PORT was specified");
    }

    let mut config = ConfigBuilder::new(ListenConfig::Ipv4 {
        ip: args.listen_ipv4,
        port: args.listen_port,
    });

    if let Some(cidr) = &args.cidr {
        let socket = UdpSocket::bind("0.0.0.0:0")?;
        socket.connect("8.8.8.8:80")?;
        let local_addr = socket.local_addr()?;

        match local_addr.ip() {
            std::net::IpAddr::V4(ip) => {
                if cidr.contains(&ip) {
                    info!("Found ip {:?} within cidr: {:?}, allowing automatic discovery table addition for source addresses under this range", ip, cidr);
                    config.allowed_cidr(cidr);
                } else {
                    warn!(
                        "added --cidr flag but local ip ({:?}) is not contained within range: {:?}",
                        ip, cidr
                    )
                }
            }
            std::net::IpAddr::V6(_) => {
                warn!("allowed cidr only compatible with ipv4")
            }
        };
    }

    info!(
        "Discovery server listening on {:?}:{:?}",
        args.listen_ipv4, args.listen_port
    );
    let mut discv5: Discv5<DefaultProtocolId> = Discv5::new(
        enr,
        key,
        config
            .request_timeout(Duration::from_secs(3))
            .vote_duration(Duration::from_secs(120))
            .build(),
    )?;

    discv5
        .start()
        .await
        .expect("Should be able to start the server");

    let server_ref = Arc::new(discv5);

    stats::run(Arc::clone(&server_ref), None, 100);
    events::run(Arc::clone(&server_ref));
    echo::run(args.rpc_addr, args.rpc_port, enr_str);

    let mut sigterm = signal(SignalKind::terminate())?;
    let mut sigint = signal(SignalKind::interrupt())?;

    // Listen for shutdown signals
    info!("Listening for termination request");

    // Wait for either SIGTERM or SIGINT
    select! {
        _ = sigterm.recv() => {
            info!("Received SIGTERM - initiating graceful shutdown");
        }
        _ = sigint.recv() => {
            info!("Received SIGINT - initiating graceful shutdown");
        }
    }

    Ok(())
}

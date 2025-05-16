use std::sync::Arc;

use discv5::{Discv5, Event};
use tracing::info;

pub fn run(server: Arc<Discv5>) {
    tokio::spawn(async move {
        let mut event_stream = server.event_stream().await.unwrap();
        loop {
            match event_stream.recv().await {
                Some(Event::SocketUpdated(addr)) => {
                    info!("Nodes ENR socket address has been updated to: {:?}", addr);
                }
                Some(Event::Discovered(enr)) => {
                    info!("A peer has been discovered: {}", enr.node_id());
                }
                Some(Event::UnverifiableEnr { enr, .. }) => {
                    info!(
                        "A peer has been added to the routing table with enr: {}",
                        enr
                    );
                }
                Some(Event::NodeInserted { node_id, .. }) => {
                    info!(
                        "A peer has been added to the routing table with node_id: {}",
                        node_id
                    );
                }
                Some(Event::SessionEstablished(enr, addr)) => {
                    info!(
                        "A session has been established with peer: {} at address: {}",
                        enr, addr
                    );
                }
                Some(Event::TalkRequest(talk_request)) => {
                    info!(
                        "A talk request has been received from peer: {}",
                        talk_request.node_id()
                    );
                }
                _ => {}
            }
        }
    });
}

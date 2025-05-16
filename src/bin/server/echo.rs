use std::net::Ipv4Addr;

use tokio::{io::AsyncWriteExt, net::TcpListener};
use tracing::{error, info};

pub fn run(addr: Ipv4Addr, port: u16, enr: &'static str) {
    tokio::spawn(async move {
        let addr = format!("{}:{}", addr, port);
        let listener = TcpListener::bind(&addr)
            .await
            .expect("could not bind ENR echo server");
        info!("ENR echo server running on {:?}", addr);

        let response = format!(
            "HTTP/1.1 200 OK\r\n\
        Content-Type: text/plain\r\n\
        Content-Length: {}\r\n\
        Connection: close\r\n\
        \r\n\
        {}",
            enr.len(),
            enr
        );
        loop {
            match listener.accept().await {
                Ok((mut socket, _)) => {
                    let response = response.clone();
                    tokio::spawn(async move {
                        if let Err(e) = socket.write_all(response.as_bytes()).await {
                            error!("Failed to write to socket: {}", e);
                        }
                    });
                }
                Err(e) => error!("Failed to accept connection: {}", e),
            }
        }
    });
}

lazy_static::lazy_static! {
    /// Entry port to the proxy.
    static ref ENTRY: &'static str = {
        if crate::TARGET_REPLACEMENT.0 == b":9223" {
            "0.0.0.0:9222"
        } else {
            "0.0.0.0:9223"
        }
    };
    /// Target chrome server.
    static ref TARGET: &'static str = {
        if crate::TARGET_REPLACEMENT.1 == b":9222" {
            "0.0.0.0:9223"
        } else {
            "0.0.0.0:9224"
        }
    };
    /// The buffer size.
    static ref BUFFER_SIZE: usize = {
        let buffer_size = std::env::var("BUFFER_SIZE")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(131072); // Default to 128kb
        buffer_size
    };
}

pub(crate) mod proxy {
    use std::net::SocketAddr;
    use tokio::net::{TcpListener, TcpStream};

    pub async fn run_proxy() -> std::io::Result<()> {
        let listener = TcpListener::bind(*crate::proxy::ENTRY).await?;
        println!("Proxy Listening on {}", *crate::proxy::ENTRY);

        loop {
            let (mut client_stream, client_addr) = listener.accept().await?;

            tokio::spawn(async move {
                if let Err(err) = handle_connection(&mut client_stream, client_addr).await {
                    tracing::error!("Error handling connection: {}", err);
                }
            });
        }
    }

    async fn handle_connection(
        client_stream: &mut TcpStream,
        client_addr: SocketAddr,
    ) -> std::io::Result<()> {
        tracing::info!("Accepted connection from {}", client_addr);

        match TcpStream::connect(*crate::proxy::TARGET).await {
            Ok(mut server_stream) => {
                tokio::io::copy_bidirectional_with_sizes(
                    client_stream,
                    &mut server_stream,
                    *crate::proxy::BUFFER_SIZE,
                    *crate::proxy::BUFFER_SIZE,
                )
                .await?;

                Ok(())
            }
            Err(e) => {
                tracing::error!("Failed to connect to server: {}", e);
                Err(e)
            }
        }
    }
}

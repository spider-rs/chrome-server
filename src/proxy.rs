pub(crate) mod proxy {
    use std::net::SocketAddr;
    use tokio::net::{TcpListener, TcpStream};

    pub async fn run_proxy() -> std::io::Result<()> {
        let listener = TcpListener::bind("0.0.0.0:9222").await?;
        tracing::info!("Proxy Listening on 127.0.0.1:9222");

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
        lazy_static! {
            static ref BUFFER_SIZE: usize = {
                let buffer_size = std::env::var("BUFFER_SIZE")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(131072); // Default to 128kb
                buffer_size
            };
        }

        match TcpStream::connect("127.0.0.1:9223").await {
            Ok(mut server_stream) => {
                tokio::io::copy_bidirectional_with_sizes(
                    client_stream,
                    &mut server_stream,
                    *BUFFER_SIZE,
                    *BUFFER_SIZE,
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

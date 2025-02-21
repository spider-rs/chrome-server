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
    use std::time::Duration;

    use tokio::{
        // io::AsyncWriteExt,
        net::{TcpListener, TcpStream},
    };

    pub async fn run_proxy() -> std::io::Result<()> {
        let listener = TcpListener::bind(*crate::proxy::ENTRY).await?;
        println!("Proxy Listening on {}", *crate::proxy::ENTRY);

        loop {
            let (mut client_stream, client_addr) = listener.accept().await?;
            tracing::info!("Accepted connection from {}", client_addr);

            tokio::spawn(async move {
                if let Err(err) = handle_connection(&mut client_stream).await {
                    tracing::error!("Error handling connection: {}", err);
                }
            });
        }
    }
    async fn handle_connection(client_stream: &mut TcpStream) -> std::io::Result<()> {
        let mut server_stream: Option<TcpStream> = None;
        let mut attempts = 0;

        while attempts < 10 && server_stream.is_none() {
            match TcpStream::connect(*crate::proxy::TARGET).await {
                Ok(stream) => {
                    if let Ok(err) = stream.take_error() {
                        if err.is_none() {
                            if let Some(e) = err {
                                tracing::error!("Err: {:?}", e)
                            }
                            server_stream = Some(stream);
                            break;
                        } else {
                            tracing::error!("{:?}", err)
                        }
                    } else {
                        server_stream = Some(stream);
                        break;
                    }
                }
                Err(e) => {
                    tokio::time::sleep(Duration::from_millis(200)).await;
                    tracing::warn!(
                        "Failed to connect to server (attempt {}): {}",
                        attempts + 1,
                        e
                    );
                    attempts += 1;
                }
            }
        }

        if let Some(mut server_stream) = server_stream {
            tokio::io::copy_bidirectional_with_sizes(
                client_stream,
                &mut server_stream,
                *crate::proxy::BUFFER_SIZE,
                *crate::proxy::BUFFER_SIZE,
            )
            .await?;

            // server_stream.flush().await?;
            // client_stream.flush().await?;

            Ok(())
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Failed to connect after several attempts",
            ))
        }
    }
}

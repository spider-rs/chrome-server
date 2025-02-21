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
    use std::{io::ErrorKind, time::Duration};

    use tokio::{
        // io::AsyncWriteExt,
        net::{TcpListener, TcpStream},
    };

    use crate::{fork, shutdown_instances};

    pub async fn run_proxy() -> std::io::Result<()> {
        let listener = TcpListener::bind(*crate::proxy::ENTRY).await?;
        println!("Proxy Listening on {}", *crate::proxy::ENTRY);

        loop {
            let (mut client_stream, client_addr) = listener.accept().await?;
            tracing::info!("Accepted connection from {}", client_addr);

            tokio::spawn(async move {
                let mut should_retry = false;

                if let Err(err) = handle_connection(&mut client_stream).await {
                    if err.kind() == ErrorKind::NotConnected || err.kind() == ErrorKind::Other {
                        should_retry = true;
                        tracing::error!("Error handling connection: {}. Restarting Chrome.", err);
                        // send a signal instead or channel to prevent race
                        shutdown_instances().await;
                        fork(Some(*crate::DEFAULT_PORT)).await;
                    } else {
                        // ignore connection resets by peer
                        if err.kind() != ErrorKind::ConnectionReset {
                            tracing::error!("Error handling connection: {}", err);
                        }
                    }
                }

                if should_retry {
                    tokio::task::yield_now().await;
                    let _ = handle_connection(&mut client_stream).await;
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
                    server_stream = Some(stream);
                }
                Err(e) => {
                    tokio::time::sleep(Duration::from_millis(250)).await;
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

            Ok(())
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Failed to connect after several attempts",
            ))
        }
    }
}

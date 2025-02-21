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
    use std::io::ErrorKind;
    use tokio::{
        io::{AsyncReadExt, AsyncWriteExt},
        net::{TcpListener, TcpStream},
    };

    use crate::{connect_with_retries, fork, shutdown_instances};

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
        let server_stream: Option<TcpStream> = connect_with_retries(*crate::proxy::TARGET).await;

        if let Some(mut server_stream) = server_stream {
            let buffer_size = *crate::proxy::BUFFER_SIZE;
            let mut buf1 = vec![0u8; buffer_size].into_boxed_slice();
            let mut buf2 = vec![0u8; buffer_size].into_boxed_slice();

            loop {
                tokio::select! {
                    a = server_stream.read(&mut buf1) => {
                        let size = match a{
                            Ok(p) => p,
                            Err(_) => break,
                        };
                        if size == 0 {
                            break;
                        }
                        match client_stream.write_all(&buf1[..size]).await{
                            Ok(_) => {},
                            Err(_) => break,
                        };
                    },
                    b = client_stream.read(&mut buf2) => {
                        let size = match b{
                            Ok(p) => p,
                            Err(_) => break,
                        };
                        if size == 0 {
                            break;
                        }
                        match server_stream.write_all(&buf2[..size]).await{
                            Ok(_) => {},
                            Err(_) => break,
                        };
                    },
                    else => {
                        break;
                    }
                }
            }

            Ok(())
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Failed to connect after several attempts",
            ))
        }
    }
}

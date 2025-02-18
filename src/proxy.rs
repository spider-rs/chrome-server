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
    /// Re-use the connection socket.
    static ref REUSE_SOCKET: bool  = {
        std::env::var("REUSE_SOCKET").unwrap_or_default() == "true"
    };
}

pub(crate) mod proxy {
    use std::time::Duration;

    use tokio::net::{TcpListener, TcpStream};

    /// Run the direct proxy connection.
    pub async fn run_proxy_direct() -> std::io::Result<()> {
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

    #[cfg(target_os = "linux")]
    /// Keep a direct connection to the server.
    pub async fn run_proxy_io() -> std::io::Result<()> {
        let listener = TcpListener::bind(*crate::proxy::ENTRY).await?;
        let std_stream = std::net::TcpStream::connect(*crate::proxy::TARGET)?;

        std_stream.set_nonblocking(true)?;
        std_stream.set_nodelay(true)?;

        println!("Proxy(REUSE) Listening on {}", *crate::proxy::ENTRY);

        loop {
            let (mut client_stream, client_addr) = listener.accept().await?;
            tracing::info!("Accepted connection from {}", client_addr);
            match std_stream.try_clone() {
                Ok(std_stream) => {
                    tokio::spawn(async move {
                        match TcpStream::from_std(std_stream) {
                            Ok(mut server_stream) => {
                                let _ = tokio::io::copy_bidirectional_with_sizes(
                                    &mut client_stream,
                                    &mut server_stream,
                                    *crate::proxy::BUFFER_SIZE,
                                    *crate::proxy::BUFFER_SIZE,
                                )
                                .await;
                            }
                            _ => {
                                if let Err(err) = handle_connection(&mut client_stream).await {
                                    tracing::error!("Error handling connection: {}", err);
                                }
                            }
                        }
                    });
                }
                _ => {
                    tokio::spawn(async move {
                        if let Err(err) = handle_connection(&mut client_stream).await {
                            tracing::error!("Error handling connection: {}", err);
                        }
                    });
                }
            }
        }
    }

    #[cfg(not(target_os = "linux"))]
    pub async fn run_proxy_io() -> std::io::Result<()> {
        run_proxy_direct().await?;
        Ok(())
    }

    pub async fn run_proxy() -> std::io::Result<()> {
        if *crate::proxy::REUSE_SOCKET {
            tokio::time::sleep(Duration::from_millis(500)).await;
            if let Err(_) = run_proxy_io().await {
                run_proxy_direct().await?;
            }
        } else {
            run_proxy_direct().await?;
        }

        Ok(())
    }

    async fn handle_connection(client_stream: &mut TcpStream) -> std::io::Result<()> {
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

pub(crate) mod proxy {
    use crate::conf::{BUFFER_SIZE, ENTRY, TARGET, TEN_SECONDS};
    use crate::{connect_with_retries, fork, shutdown_instances, CACHEABLE, LAST_CACHE};
    use std::{io::ErrorKind, time::Instant};
    use tokio::{
        io::{AsyncReadExt, AsyncWriteExt},
        net::{TcpListener, TcpStream},
    };

    /// Run the proxy forwarder for chrome. This allows connecting to chrome outside of the network.
    pub async fn run_proxy() -> std::io::Result<()> {
        let listener = TcpListener::bind(*ENTRY).await?;
        println!("Proxy Listening on {}", *ENTRY);
        let base_time = Instant::now();

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
                        // todo: we need to swap to a new port and use a LB to track the drain to reset the ports used.
                        shutdown_instances().await;
                        fork(Some(*crate::DEFAULT_PORT)).await;
                        CACHEABLE.store(false, std::sync::atomic::Ordering::Relaxed);
                        LAST_CACHE.store(
                            base_time.elapsed().as_secs().try_into().unwrap_or_default(),
                            std::sync::atomic::Ordering::Relaxed,
                        );
                    } else {
                        // ignore connection resets by peer
                        if err.kind() != ErrorKind::ConnectionReset {
                            tracing::error!("Error handling connection: {}", err);
                        }
                    }
                } else if !CACHEABLE.load(std::sync::atomic::Ordering::Relaxed) {
                    let elasped = LAST_CACHE.load(std::sync::atomic::Ordering::Relaxed);

                    if elasped > 0 {
                        let elapsed_since_base = base_time.elapsed();
                        let total_elapsed =
                            tokio::time::Duration::from_secs(elasped) + elapsed_since_base;

                        if total_elapsed >= *TEN_SECONDS {
                            CACHEABLE.store(true, std::sync::atomic::Ordering::Relaxed);
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

    /// Handle the proxy connection.
    async fn handle_connection(client_stream: &mut TcpStream) -> std::io::Result<()> {
        let server_stream: Option<TcpStream> = connect_with_retries(*TARGET).await;

        if let Some(mut server_stream) = server_stream {
            let buffer_size = *BUFFER_SIZE;
            let mut buf1 = vec![0u8; buffer_size];
            let mut buf2 = vec![0u8; buffer_size];

            loop {
                tokio::select! {
                    a = server_stream.read(&mut buf1) => {
                        let size = match a {
                            Ok(p) => p,
                            Err(_) => break,
                        };
                        if size == 0 {
                            break;
                        }
                        if let Err(_) = client_stream.write_all(&buf1[..size]).await  {
                            break;
                        }
                    },
                    b = client_stream.read(&mut buf2) => {
                        let size = match b {
                            Ok(p) => p,
                            Err(_) => break,
                        };
                        if size == 0 {
                            break;
                        }
                        if let Err(_) = server_stream.write_all(&buf2[..size]).await {
                            break;
                        }
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

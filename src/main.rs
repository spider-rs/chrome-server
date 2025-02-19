#[macro_use]
extern crate lazy_static;
use cached::proc_macro::once;

mod conf;
mod modify;
mod proxy;

use conf::{
    CHROME_ADDRESS, CHROME_ARGS, CHROME_INSTANCES, CHROME_PATH, DEFAULT_PORT, DEFAULT_PORT_SERVER,
    ENDPOINT, HOST_NAME, IS_HEALTHY, LIGHTPANDA_ARGS, LIGHT_PANDA, TARGET_REPLACEMENT,
};
use core::sync::atomic::Ordering;
use http_body_util::Full;
use hyper::{
    body::{Bytes, Incoming},
    server::conn::http1,
    service::service_fn,
    Method, Request, Response, StatusCode,
};
use hyper_util::rt::TokioIo;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::process::Command;
use tokio::{
    net::{TcpListener, TcpStream},
    signal,
};

use std::time::Duration;
use tokio::time::{sleep, timeout};

/// Attempt the connection.
async fn connect_with_retries(address: &str) -> Option<TcpStream> {
    let mut attempts = 0;
    loop {
        match timeout(Duration::from_secs(1), TcpStream::connect(address)).await {
            Ok(Ok(stream)) => return Some(stream),
            Ok(Err(e)) => {
                attempts += 1;
                tracing::error!("Failed to connect: {}. Attempt {} of 3", e, attempts);
            }
            Err(_) => {
                attempts += 1;
                tracing::error!("Connection attempt timed out. Attempt {} of 3", attempts);
            }
        }

        if attempts >= 3 {
            return None;
        }

        sleep(Duration::from_millis(1500)).await;
    }
}

/// Shutdown the chrome instance by process id
#[cfg(target_os = "windows")]
fn shutdown(pid: &u32) {
    let _ = Command::new("taskkill")
        .args(["/PID", &pid.to_string(), "/F"])
        .spawn();
}

/// Shutdown the chrome instance by process id
#[cfg(not(target_os = "windows"))]
fn shutdown(pid: &u32) {
    let _ = Command::new("kill").args(["-9", &pid.to_string()]).spawn();
}

/// Fork a chrome process
async fn fork(port: Option<u32>) -> String {
    let id = if !*LIGHT_PANDA {
        let mut command = Command::new(&*CHROME_PATH);
        let mut chrome_args = CHROME_ARGS.map(|e| e.to_string());

        if !CHROME_ADDRESS.is_empty() {
            chrome_args[0] = format!("--remote-debugging-address={}", &CHROME_ADDRESS.to_string());
        }

        if let Some(port) = port {
            chrome_args[1] = format!("--remote-debugging-port={}", &port.to_string());
        }

        let cmd = command.args(&chrome_args);

        let id = if let Ok(child) = cmd.spawn() {
            let cid = child.id();

            tracing::info!("Chrome PID: {}", cid);

            cid
        } else {
            tracing::error!("chrome command didn't start");
            0
        };

        id
    } else {
        let panda_args = LIGHTPANDA_ARGS.map(|e| e.to_string());
        let mut command = Command::new(&*CHROME_PATH);

        let host = panda_args[0].replace("--host=", "");
        let port = panda_args[1].replace("--port=", "");

        let id = if let Ok(child) = command
            .args(["--port", &port])
            .args(["--host", &host])
            .spawn()
        {
            let cid = child.id();

            tracing::info!("Chrome PID: {}", cid);

            cid
        } else {
            tracing::error!("chrome command didn't start");
            0
        };

        id
    };

    CHROME_INSTANCES.lock().await.insert(id.into());

    id.to_string()
}

/// Get json endpoint for chrome instance proxying
#[once(option = true, sync_writes = true)]
async fn version_handler_bytes(endpoint_path: Option<&str>) -> Option<Bytes> {
    use http_body_util::BodyExt;

    let url = endpoint_path
        .unwrap_or(&ENDPOINT.as_str())
        .parse::<hyper::Uri>()
        .expect("valid chrome endpoint");

    let req = Request::builder()
        .method(Method::GET)
        .uri("/json/version")
        .header(
            hyper::header::HOST,
            url.authority()
                .map_or_else(|| "localhost".to_string(), |f| f.as_str().to_string()),
        )
        .header(hyper::header::CONTENT_TYPE, "application/json")
        .body(http_body_util::Empty::<Bytes>::new())
        .expect("Failed to build the request");

    let host = url.host().expect("uri has no host");
    let port = url.port_u16().unwrap_or(80);

    let address = format!("{}:{}", host, port);

    let resp = if let Some(stream) = connect_with_retries(&address).await {
        let io = TokioIo::new(stream);

        if let Ok((mut client, conn)) = hyper::client::conn::http1::handshake(io).await {
            tokio::task::spawn(async move {
                if let Err(err) = conn.await {
                    tracing::error!("Connection failed: {:?}", err);
                }
            });

            match client.send_request(req).await {
                Ok(mut resp) => {
                    IS_HEALTHY.store(true, Ordering::Relaxed);

                    let mut bytes_mut = vec![];

                    while let Some(next) = resp.frame().await {
                        if let Ok(frame) = next {
                            if let Some(chunk) = frame.data_ref() {
                                bytes_mut.extend(chunk);
                            }
                        }
                    }

                    if !HOST_NAME.is_empty() {
                        let body = modify::modify_json_output(bytes_mut.into());
                        Some(body)
                    } else {
                        Some(bytes_mut.into())
                    }
                }
                _ => {
                    IS_HEALTHY.store(false, Ordering::Relaxed);
                    None
                }
            }
        } else {
            None
        }
    } else {
        None
    };

    resp
}

/// Health check handler
async fn health_check_handler() -> Result<Response<Full<Bytes>>, Infallible> {
    if IS_HEALTHY.load(Ordering::Relaxed) {
        Ok(Response::new(Full::new(Bytes::from("healthy"))))
    } else {
        let mut response = Response::new(Full::new(Bytes::from("unhealthy")));
        *response.status_mut() = StatusCode::SERVICE_UNAVAILABLE;

        Ok(response)
    }
}

/// Fork handler
async fn fork_handler(port: Option<u32>) -> Result<Response<Full<Bytes>>, Infallible> {
    let pid = fork(port).await;
    let pid = format!("Forked process with pid: {}", pid);

    Ok(Response::new(Full::new(Bytes::from(pid))))
}

/// Shutdown handler
async fn shutdown_handler() -> Result<Response<Full<Bytes>>, Infallible> {
    let mut mutx = CHROME_INSTANCES.lock().await;
    for pid in mutx.iter() {
        shutdown(pid);
    }
    mutx.clear();

    Ok(Response::new(Full::new(Bytes::from(
        "Shutdown successful.",
    ))))
}

/// Request handler
async fn request_handler(req: Request<Incoming>) -> Result<Response<Full<Bytes>>, Infallible> {
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/health") => health_check_handler().await,
        (&Method::GET, "/") => health_check_handler().await,
        (&Method::POST, "/fork") => fork_handler(None).await,
        (&Method::POST, path) if path.starts_with("/fork/") => {
            if let Some(port) = path.split('/').nth(2) {
                if let Ok(port) = port.parse::<u32>() {
                    fork_handler(Some(port)).await
                } else {
                    let message = Response::new(Full::new(Bytes::from("Invalid port argument")));

                    Ok(message)
                }
            } else {
                let message = Response::new(Full::new(Bytes::from("Invalid path")));

                Ok(message)
            }
        }
        (&Method::GET, "/json/version") => {
            let body = version_handler_bytes(None).await.unwrap_or_default();
            let empty = body.is_empty();

            let mut resp = Response::new(Full::new(body));

            resp.headers_mut().insert(
                hyper::header::CONTENT_TYPE,
                hyper::header::HeaderValue::from_static("application/json"),
            );

            if empty {
                *resp.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
            }

            Ok(resp)
        }
        (&Method::POST, "/shutdown") => shutdown_handler().await,
        _ => {
            let mut resp = Response::new(Full::new(Bytes::from("Not Found")));

            *resp.status_mut() = StatusCode::NOT_FOUND;

            Ok(resp)
        }
    }
}

/// Main entry for the proxy.
async fn run_main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let auto_start = std::env::args().nth(3).unwrap_or_else(|| {
        let auto = std::env::var("CHROME_INIT").unwrap_or("true".into());
        if auto == "true" {
            "init".into()
        } else {
            "ignore".into()
        }
    });

    if auto_start == "init" {
        fork(Some(*DEFAULT_PORT)).await;
    }

    let addr = SocketAddr::new(
        std::net::IpAddr::V4(std::net::Ipv4Addr::new(0, 0, 0, 0)),
        *DEFAULT_PORT_SERVER,
    );

    let listener = TcpListener::bind(addr).await.expect("connection");

    let make_svc = async move {
        loop {
            if let Ok((tcp, _)) = listener.accept().await {
                let io = TokioIo::new(tcp);

                tokio::task::spawn(async move {
                    if let Err(err) = http1::Builder::new()
                        .preserve_header_case(true)
                        .title_case_headers(true)
                        .serve_connection(io, service_fn(request_handler))
                        .await
                    {
                        eprintln!("Error serving connection: {:?}", err);
                    }
                });
            }
        }
    };

    println!(
        "Chrome server running on {}:{}",
        if CHROME_ADDRESS.is_empty() {
            "localhost"
        } else {
            &CHROME_ADDRESS
        },
        DEFAULT_PORT_SERVER.to_string()
    );

    tokio::select! {
        _ = make_svc => Ok(()),
        _ = crate::proxy::proxy::run_proxy() =>  Ok(()),
        _ = signal::ctrl_c() => Ok(()),
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    run_main().await
}

#[macro_use]
extern crate lazy_static;

mod conf;
mod modify;
mod proxy;

use conf::{
    CHROME_ARGS, CHROME_INSTANCES, CLIENT, DEFAULT_PORT, DEFAULT_PORT_SERVER, IS_HEALTHY,
    TARGET_REPLACEMENT,
};
use core::sync::atomic::Ordering;
use hyper::{Body, Method, Request};
use std::io::{self, Read};
use std::process::{Command, Stdio};
use tokio::{signal, sync::oneshot};
use warp::{Filter, Rejection, Reply};

type Result<T> = std::result::Result<T, Rejection>;

lazy_static::lazy_static! {
    /// The hostname of the machine to replace 127.0.0.1 when making request to /json/version on port 6000.
    pub static ref HOST_NAME: String = {
        let mut hostname = String::new();

        if let Ok(name) = std::env::var("HOSTNAME_OVERRIDE") {
            hostname = name;
        }

        if hostname.is_empty() {
            if let Ok(name) = std::env::var("HOSTNAME") {
                hostname = name;
            }
        }

        hostname
    };
    static ref ENDPOINT: String = {
        let default_port = std::env::args()
            .nth(4)
            .unwrap_or("9223".into())
            .parse::<u32>()
            .unwrap_or_default();
        let default_port = if default_port == 0 {
            9223
        } else {
            default_port
        };
        format!("http://127.0.0.1:{}/json/version", default_port)
    };

    /// Old headless error.
    static ref OLD_HEADLESS: &'static str = r#"Old Headless mode has been removed from the Chrome binary. Please use the new Headless mode (https://developer.chrome.com/docs/chromium/new-headless) or the chrome-headless-shell which is a standalone implementation of the old Headless mode (https://developer.chrome.com/blog/chrome-headless-shell)."#;
}

/// shutdown the chrome instance by process id
#[cfg(target_os = "windows")]
fn shutdown(pid: &u32) {
    let _ = Command::new("taskkill")
        .args(["/PID", &pid.to_string(), "/F"])
        .spawn();
}

/// shutdown the chrome instance by process id
#[cfg(not(target_os = "windows"))]
fn shutdown(pid: &u32) {
    let _ = Command::new("kill").args(["-9", &pid.to_string()]).spawn();
}

/// fork a chrome process
fn fork(chrome_path: &String, chrome_address: &String, port: Option<u32>) -> String {
    let mut command = Command::new(chrome_path);
    let mut chrome_args = CHROME_ARGS.map(|e| e.to_string());

    if !chrome_address.is_empty() {
        chrome_args[0] = format!("--remote-debugging-address={}", &chrome_address.to_string());
    }

    if let Some(port) = port {
        chrome_args[1] = format!("--remote-debugging-port={}", &port.to_string());
    }

    let id = if let Ok(mut child) = command
        .args(&chrome_args)
        .stderr(Stdio::piped()) // Capture stdout
        .spawn()
    {
        let cid = child.id();

        if let Some(stdout) = child.stderr.take() {
            let mut reader = io::BufReader::new(stdout);
            let mut output = String::new();

            if reader.read_to_string(&mut output).is_ok() {

                if output.trim() == *OLD_HEADLESS {
                    tracing::info!(
                        "Chrome PID: {} failed to spawn. Attempting headless='new'",
                        cid
                    );

                    chrome_args[2] = format!("--headless={}", &"new");

                    if let Ok(child) = command.args(&chrome_args).spawn() {
                        let cid = child.id();
                        tracing::info!("Chrome PID: {}", cid);

                        if let Ok(mut mutx) = CHROME_INSTANCES.lock() {
                            mutx.insert(cid.to_owned());
                        }

                        return cid.to_string();
                    }
                } else {
                    tracing::error!("Chrome failed to spawn {}", output);
                }
            }
        }

        tracing::info!("Chrome PID: {}", cid);

        if let Ok(mut mutx) = CHROME_INSTANCES.lock() {
            mutx.insert(cid.to_owned());
        }

        cid
    } else {
        tracing::error!("chrome command didn't start");
        0
    }
    .to_string();

    id
}

/// get json endpoint for chrome instance proxying
async fn version_handler(endpoint_path: Option<&str>) -> Result<impl Reply> {
    use hyper::body::HttpBody;
    let req = Request::builder()
        .method(Method::GET)
        .uri(endpoint_path.unwrap_or(ENDPOINT.as_str()))
        .header("content-type", "application/json")
        .body(Body::default())
        .unwrap_or_default();

    let resp = match CLIENT.request(req).await {
        Ok(mut resp) => {
            if !IS_HEALTHY.load(Ordering::Relaxed) {
                IS_HEALTHY.store(true, Ordering::Relaxed);
            }

            if !HOST_NAME.is_empty() {
                if let Ok(body_bytes) = resp.body_mut().collect().await {
                    let body = modify::modify_json_output(body_bytes.to_bytes());

                    let modified_response = hyper::Response::builder()
                        .status(resp.status())
                        .body(Body::from(body))
                        .unwrap_or_default();

                    return Ok(modified_response);
                }
            }

            resp
        }
        _ => {
            IS_HEALTHY.store(false, Ordering::Relaxed);
            Default::default()
        }
    };

    Ok(resp)
}

/// get json endpoint for chrome instance proxying
async fn version_handler_with_path(port: u32) -> Result<impl Reply> {
    let req = Request::builder()
        .method(Method::GET)
        .uri(format!("http://127.0.0.1:{}/json/version", port))
        .header("content-type", "application/json")
        .body(Body::default())
        .unwrap_or_default();

    let resp = match CLIENT.request(req).await {
        Ok(resp) => resp,
        _ => Default::default(),
    };

    Ok(resp)
}

/// health check server
async fn hc() -> Result<impl Reply> {
    use hyper::Response;
    use hyper::StatusCode;

    #[derive(Debug)]
    struct HealthCheckError;
    impl warp::reject::Reject for HealthCheckError {}

    if IS_HEALTHY.load(Ordering::Relaxed) {
        Response::builder()
            .status(StatusCode::OK)
            .body("healthy!")
            .map_err(|_e| warp::reject::custom(HealthCheckError))
    } else {
        Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body("unhealthy!")
            .map_err(|_e| warp::reject::custom(HealthCheckError))
    }
}

/// Get the default chrome bin location per OS.
fn get_default_chrome_bin() -> &'static str {
    if cfg!(target_os = "windows") {
        "chrome.exe"
    } else if cfg!(target_os = "macos") {
        "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome"
    } else if cfg!(target_os = "linux") {
        "chromium"
    } else {
        "chrome"
    }
}

/// Main entry for the proxy.
async fn run_main() {
    let chrome_path = std::env::args().nth(1).unwrap_or_else(|| {
        std::env::var("CHROME_PATH").unwrap_or_else(|_| get_default_chrome_bin().to_string())
    });
    let chrome_address = std::env::args().nth(2).unwrap_or("127.0.0.1".to_string());
    let auto_start = std::env::args().nth(3).unwrap_or_default();
    let chrome_path_1 = chrome_path.clone();
    let chrome_address_1 = chrome_address.clone();
    let chrome_address_2 = chrome_address.clone();

    // init chrome process
    if auto_start == "init" {
        fork(&chrome_path, &chrome_address_1, Some(*DEFAULT_PORT));
    }

    let health_check = warp::path::end()
        .and_then(hc)
        .with(warp::cors().allow_any_origin());

    let chrome_init = move || fork(&chrome_path, &chrome_address_1, None);
    let chrome_init_args = move |port: u32| fork(&chrome_path_1, &chrome_address_2, Some(port));
    let json_args = move || version_handler(None);
    let json_args_with_port = move |port| version_handler_with_path(port);

    let fork = warp::path!("fork").map(chrome_init);
    let fork_with_port = warp::path!("fork" / u32).map(chrome_init_args);

    let version = warp::path!("json" / "version").and_then(json_args);
    let version_with_port = warp::path!("json" / "version" / u32).and_then(json_args_with_port);

    let shutdown_fn = warp::path!("shutdown" / u32).map(|cid: u32| {
        let shutdown_id = match CHROME_INSTANCES.lock() {
            Ok(mutx) => {
                let pid = mutx.get(&cid);

                match pid {
                    Some(pid) => {
                        shutdown(pid);
                        pid.to_string()
                    }
                    _ => "0".into(),
                }
            }
            _ => "0".into(),
        };

        shutdown_id
    });

    let shutdown_base_fn = || {
        if let Ok(mutx) = CHROME_INSTANCES.lock() {
            for pid in mutx.iter() {
                shutdown(pid);
            }
        }

        "0"
    };

    let shutdown_base_fn = warp::path!("shutdown").map(shutdown_base_fn);

    let ctrls = warp::post().and(fork.with(warp::cors().allow_any_origin()));
    let ctrls_fork = warp::post().and(fork_with_port.with(warp::cors().allow_any_origin()));
    let shutdown = warp::post().and(shutdown_fn.with(warp::cors().allow_any_origin()));
    let shutdown_base = warp::post().and(shutdown_base_fn.with(warp::cors().allow_any_origin()));
    let version_port = warp::post().and(version_with_port.with(warp::cors().allow_any_origin()));

    let routes = warp::get()
        .and(health_check)
        .or(shutdown)
        .or(shutdown_base)
        .or(version)
        .or(ctrls_fork)
        .or(version_port)
        .or(ctrls);

    println!(
        "Chrome server at {}:{}",
        if chrome_address.is_empty() {
            "localhost"
        } else {
            &chrome_address
        },
        DEFAULT_PORT_SERVER.to_string()
    );

    let (tx_shutdown, rx_shutdown) = oneshot::channel::<()>();

    let signal_handle = tokio::spawn(async move {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to setup signal handler")
            .recv()
            .await;

        tracing::info!("Received termination signal");

        if tx_shutdown.send(()).is_err() {
            tracing::error!("Failed to send shutdown signal through channel");
        }
    });

    let srv = async {
        tokio::select! {
            _ = warp::serve(routes)
            .run(([0, 0, 0, 0], DEFAULT_PORT_SERVER.to_owned())) => tracing::error!("Server finished without external shutdown."),
            _ = proxy::proxy::run_proxy() => tracing::error!("Received shutdown signal."),
            _ = rx_shutdown =>  tracing::error!("Received shutdown signal."),
        }
    };

    tokio::select! {
        _ = srv => (),
        _ = signal_handle => (),
    }
}

#[tokio::main]
async fn main() {
    run_main().await
}

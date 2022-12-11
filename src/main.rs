use hyper::{Body, Client, Method, Request};
use std::process::Command;
use warp::{Filter, Rejection, Reply};

type Result<T> = std::result::Result<T, Rejection>;

/// static chrome arguments to start application
static CHROME_ARGS: [&'static str; 26] = [
    "--headless",
    "--no-sandbox",
    "--no-first-run",
    "--remote-debugging-address=0.0.0.0",
    "--remote-debugging-port=9222",
    "--max-wait-for-load=2500",
    "--hide-scrollbars",
    "--allow-running-insecure-content",
    "--autoplay-policy=user-gesture-required",
    "--ignore-certificate-errors",
    "--no-default-browser-check",
    "--disable-default-apps",
    "--disable-storage-reset",
    "--disable-dev-shm-usage",
    "--disable-domain-reliability",
    "--disable-component-update",
    "--disable-background-timer-throttling",
    "--disable-accelerated-2d-canvas",
    "--disable-accelerated-video-decode",
    "--disable-extensions",
    "--disable-popup-blocking",
    "--disable-setuid-sandbox",
    "--disable-features=ScriptStreaming,TranslateUI,BlinkGenPropertyTrees",
    "--disable-backgrounding-occluded-windows",
    "--disable-component-extensions-with-background-pages",
    "--enable-background-thread-pool",
];

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let base_route = warp::path::end().and_then(handler);
    let fork = warp::path("fork").map(|| {
        let mut command = Command::new("chromium-browser");

        let id = if let Ok(child) = command.args(CHROME_ARGS).spawn() {
            let cid = child.id();
            println!("CID is {}", child.id());

            cid
        } else {
            println!("chrome command didn't start");

            0
        }
        .to_string();

        id
    });

    let ctrls = warp::post().and(fork.with(warp::cors().allow_any_origin()));

    let routes = warp::get()
        .and(base_route.with(warp::cors().allow_any_origin()))
        .or(ctrls);

    println!("Chrome server at localhost:6000");
    warp::serve(routes).run(([0, 0, 0, 0], 6000)).await;
}

/// get json endpoint for chrome instance
async fn handler() -> Result<impl Reply> {
    let req = Request::builder()
        .method(Method::GET)
        .uri("http://127.0.0.1:9222/json/version")
        .header("content-type", "application/json")
        .body(Body::default())
        .unwrap_or_default();

    let client = Client::new();
    let resp = client.request(req).await.unwrap_or_default();

    Ok(resp)
}

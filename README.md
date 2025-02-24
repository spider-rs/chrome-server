# headless-browser

Headless Browser with Proxy and Server.

## Installation

Make sure to have Rust installed.

`cargo install headless_browser`

## Usage

1. Runs the latest browser instance with remote proxy connections and a server that can rewrite json/version for networking.
1. Can spawn and shutdown multiple headless instances manually and automatically on errors.
1. Get CDP ws connections and status (caches values for perf).

The current instance binds chrome to 0.0.0.0 when starting via API.

Use the env variable `REMOTE_ADDRESS` to change the address of the chrome instance between physical or network.

The application will pass lb health checks when using port `6000` to get the status of the chromium container.

A side loaded application is required to run chromium on a load balancer, one of the main purposes of the server.

The default port is `9223` for chromium and `9222` for the TCP proxy to connect to the instance due to `0.0.0.0` not being exposed on latest `HeadlessChrome/131.0.6778.139` and up.

## API

1. POST: `fork` to start a new chrome instance or use `fork/$port` with the port to startup the instance ex: `curl --location --request POST 'http://localhost:6000/fork/9223'`.
2. POST: `shutdown/$PID` to shutdown the instance. ex: `curl --location --request POST 'http://localhost:6000/shutdown/77057'`
3. POST: `/json/version` get the json info of the chrome instance to connect to web sockets ex: `curl --location --request POST 'http://localhost:6000/json/version'`.

### Curl Examples

`fork`

```sh
curl --location --request POST 'http://localhost:6000/fork'
```

`shutdown`

```sh
curl --location --request POST 'http://localhost:6000/shutdown'
```

`/json/version`

```sh
curl --location --request GET 'http://localhost:6000/json/version' \
--header 'Content-Type: application/json'

# example output
{
   "Browser": "HeadlessChrome/131.0.6778.139",
   "Protocol-Version": "1.3",
   "User-Agent": "Mozilla/5.0 (X11; Linux aarch64) AppleWebKit/537.36 (KHTML, like Gecko) HeadlessChrome/131.0.6778.139 Safari/537.36",
   "V8-Version": "13.1.201.16",
   "WebKit-Version": "537.36 (@c35bbcbd7c2775a12a3f320e05ac0022939b1a8a)",
   "webSocketDebuggerUrl": "ws://127.0.0.1:9222/devtools/browser/43e14f5a-6877-4e2f-846e-ab5801f1b6fc"
}
```

## Args

1. The first arg is the chrome application location example linux `'/opt/google/chrome/chrome'`.
2. The second arg is the chrome address `127.0.0.1`.
3. The third arg you can pass in `init` to auto start chrome on `9222`.

Example to start chrome (all params are optional):

```sh
headless_browser '/Applications/Google Chrome.app/Contents/MacOS/Google Chrome' 127.0.0.1 init
# Chrome PID: 87659
# Chrome server at localhost:6000
# DevTools listening on ws://127.0.0.1:9222/devtools/browser/c789f9e0-7f65-495d-baee-243eb454ea15
```

### Docker

You can build this image using the following:

1. Dockerfile (Alpine)
1. Dockerfile.playwright (Playwright Custom Chrome)
1. Dockerfile.headless_shell (Manual)
1. Dockerfile.brave
1. Dockerfile.headless_shell_playwright (Playwright Headless Shell)
1. Dockerfile.xvfb (Virtual Display)
1. Dockerfile.lightpanda

You need to set the env variable passed in as an arg `HOSTNAME_OVERRIDE` to override the docker container and set it to `host.docker.internal`.

#### Manual (WIP)

Use the following to build with docker.
Run the command `./build.sh` to build chrome on the machine with docker.
The build scripts are originally from [docker-headless-shell](https://github.com/chromedp/docker-headless-shell).

### ENV Variables

```sh
# the chrome path on the OS ex: CHROME_PATH=./chrome-headless-shell/mac_arm-132.0.6834.159/chrome-headless-shell-mac-arm64/chrome-headless-shell
CHROME_PATH=
# the remote address of the chrome intance
REMOTE_ADDRESS=
# use brave browser as default. Set the value to true.
BRAVE_ENABLED=
```

## Library

You can use the [lib](https://docs.rs/headless_browser_lib/latest/headless_browser_lib/) with `cargo add headless_browser_lib` control the startup and shutdown manually. Below is an example of using the [spider_chrome](https://github.com/spider-rs/headless-browser) project to run CDP commands concurrently fast.

```rust
futures-util = "0.3"
tokio = { version = "1", features = ["full"] }
spider_chrome = "2"
headless_browser_lib = "0.1"
```

```rust
/// spider_chrome is mapped to chromiumoxide since it was forked and kept the API the same.
use chromiumoxide::{browser::Browser, error::CdpError};
use futures_util::stream::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tokio::spawn(headless_browser_lib::run_main()); // spawn main server, proxy, and headless.
    tokio::time::sleep(std::time::Duration::from_millis(100)).await; // give a slight delay for now until we use a oneshot.

    let (browser, mut handler) =
         // port 6000 - main server entry for routing correctly across networks.
        Browser::connect_with_config("http://127.0.0.1:6000/json/version", Default::default())
            .await?;

    let handle = tokio::task::spawn(async move {
        while let Some(h) = handler.next().await {
            if h.is_err() {
                break;
            }
        }
    });

    let navigate_page = async |u: &str| -> Result<String, CdpError> {
        let page = browser.new_page(u).await?;
        let html = page.wait_for_navigation().await?.content().await?;
        Ok::<String, CdpError>(html)
    };

    let (spider_html, example_html) = tokio::join!(
        navigate_page("https://spider.cloud"),
        navigate_page("https://example.com")
    );

    browser.close().await?;
    let _ = handle.await;

    let spider_html = spider_html?;
    let example_html = example_html?;

    browser.close().await?;
    let _ = handle.await;

    println!("===\n{}\n{}\n===", "spider.cloud", spider_html);
    println!("===\n{}\n{}\n===", "example.com", example_html);

    Ok(())
}
```

## License

MIT

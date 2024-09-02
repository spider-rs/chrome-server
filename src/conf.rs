use hyper::Client;
use std::collections::HashSet;
use std::sync::{atomic::AtomicBool, Mutex};

lazy_static! {
    /// Is the instance healthy?
    pub static ref IS_HEALTHY: AtomicBool = AtomicBool::new(true);
    pub static ref CHROME_INSTANCES: Mutex<HashSet<u32>> = Mutex::new(HashSet::new());
    pub static ref DEFAULT_PORT: u32 = {
        let default_port = std::env::args()
            .nth(4)
            .unwrap_or("9222".into())
            .parse::<u32>()
            .unwrap_or_default();
        let default_port = if default_port == 0 {
            9222
        } else {
            default_port
        };

        default_port
    };
    pub static ref DEFAULT_PORT_SERVER: u16 = {
        let default_port = std::env::args()
            .nth(5)
            .unwrap_or("6000".into())
            .parse::<u16>()
            .unwrap_or_default();
        let default_port = if default_port == 0 {
            6000
        } else {
            default_port
        };

        default_port
    };
    pub static ref CHROME_ARGS: [&'static str; 66] = {
        let headless = std::env::args()
        .nth(6)
        .unwrap_or("true".into());

        let headless = if headless != "false" {
            match std::env::var("HEADLESS") {
                Ok(h) => {
                    if h == "new" {
                        "--headless=new"
                    } else {
                        "--headless=old"
                    }
                }
                _ => "--headless=old"
            }
        } else {
            ""
        };

        [
            // *SPECIAL*
            "--remote-debugging-address=0.0.0.0",
            "--remote-debugging-port=9222",
            // *SPECIAL*
            headless,
            "--no-first-run",
            "--no-sandbox",
            "--disable-setuid-sandbox",
            "--no-zygote",
            "--hide-scrollbars",
            // "--allow-pre-commit-input",
            "--user-data-dir=~/.config/google-chrome",
            "--allow-running-insecure-content",
            "--autoplay-policy=user-gesture-required",
            "--ignore-certificate-errors",
            "--no-default-browser-check",
            "--disable-gpu",
            "--disable-gpu-sandbox",
            "--disable-dev-shm-usage", // required or else container will crash not enough memory
            "--disable-threaded-scrolling",
            "--disable-demo-mode",
            "--disable-dinosaur-easter-egg",
            "--disable-fetching-hints-at-navigation-start",
            "--disable-site-isolation-trials",
            "--disable-web-security",
            "--disable-threaded-animation",
            "--disable-sync",
            "--disable-print-preview",
            "--disable-partial-raster",
            "--disable-in-process-stack-traces",
            "--disable-v8-idle-tasks",
            "--disable-low-res-tiling",
            "--disable-speech-api",
            "--disable-smooth-scrolling",
            "--disable-default-apps",
            "--disable-prompt-on-repost",
            "--disable-domain-reliability",
            "--disable-component-update",
            "--disable-background-timer-throttling",
            "--disable-breakpad",
            "--disable-software-rasterizer",
            "--disable-extensions",
            "--disable-popup-blocking",
            "--disable-hang-monitor",
            "--disable-image-animation-resync",
            "--disable-client-side-phishing-detection",
            "--disable-component-extensions-with-background-pages",
            "--disable-ipc-flooding-protection",
            "--disable-background-networking",
            "--disable-renderer-backgrounding",
            "--disable-field-trial-config",
            "--disable-back-forward-cache",
            "--disable-backgrounding-occluded-windows",
            // "--enable-automation",
            "--log-level=3",
            "--enable-logging=stderr",
            "--enable-features=SharedArrayBuffer,NetworkService,NetworkServiceInProcess",
            "--metrics-recording-only",
            "--use-mock-keychain",
            "--force-color-profile=srgb",
            "--mute-audio",
            "--no-service-autorun",
            "--password-store=basic",
            "--export-tagged-pdf",
            "--no-pings",
            "--use-gl=swiftshader",
            "--window-size=1920,1080",
            "--disable-vulkan-fallback-to-gl-for-testing",
            "--disable-vulkan-surface",
            "--disable-features=AudioServiceOutOfProcess,IsolateOrigins,site-per-process,ImprovedCookieControls,LazyFrameLoading,GlobalMediaControls,DestroyProfileOnBrowserClose,MediaRouter,DialMediaRouteProvider,AcceptCHFrame,AutoExpandDetailsElement,CertificateTransparencyComponentUpdater,AvoidUnnecessaryBeforeUnloadCheckSync,Translate"
        ]
    };
    pub static ref CLIENT: Client<hyper::client::HttpConnector> = {
        Client::new()
    };
}

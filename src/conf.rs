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
            .unwrap_or("9223".into())
            .parse::<u32>()
            .unwrap_or_default();
        let default_port = if default_port == 0 {
            9223
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

    /// The chrome args to use.
    pub static ref CHROME_ARGS: [&'static str; 71] = {
        let headless = std::env::args()
        .nth(6)
        .unwrap_or("true".into());

        let headless = if headless != "false" {
            match std::env::var("HEADLESS") {
                Ok(h) => {
                    if h == "false" {
                        ""
                    } else {
                        "--headless"
                    }
                }
                _ => "--headless"
            }
        } else {
            ""
        };

        let port = if DEFAULT_PORT.eq(&9223) {
            "--remote-debugging-port=9223"
        } else if DEFAULT_PORT.eq(&9224) {
            "--remote-debugging-port=9224"
        } else {
            "--remote-debugging-port=9222"
        };

        let use_gl = match std::env::var("CHROME_GL") {
            Ok(h) => {
                if h == "angle" {
                    "--use-gl=angle"
                } else {
                    "--use-gl=swiftshader"
                }
            }
            _ => "--use-gl=angle"
        };

        [
            // *SPECIAL*
            "--remote-debugging-address=0.0.0.0",
            port,
            // *SPECIAL*
            headless,
            "--no-first-run",
            "--no-sandbox",
            "--disable-setuid-sandbox",
            "--no-zygote",
            "--hide-scrollbars",
            "--allow-pre-commit-input",
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
            "--disable-checker-imaging",
            "--disable-image-animation-resync",
            "--disable-client-side-phishing-detection",
            "--disable-component-extensions-with-background-pages",
            "--disable-background-networking",
            "--disable-renderer-backgrounding",
            "--disable-field-trial-config",
            "--disable-back-forward-cache",
            "--disable-backgrounding-occluded-windows",
            "--log-level=3",
            "--enable-logging=stderr",
            "--enable-features=SharedArrayBuffer",
            "--metrics-recording-only",
            "--use-mock-keychain",
            "--force-color-profile=srgb",
            "--mute-audio",
            "--no-service-autorun",
            "--password-store=basic",
            "--export-tagged-pdf",
            "--no-pings",
            use_gl,
            "--window-size=1400,820",
            "--disable-vulkan-fallback-to-gl-for-testing",
            "--disable-vulkan-surface",
            "--disable-webrtc",
            "--disable-blink-features=AutomationControlled",
            "--disable-ipc-flooding-protection",
            "--blink-settings=primaryHoverType=2,availableHoverTypes=2,primaryPointerType=4,availablePointerTypes=4",
            "--disable-features=InterestFeedContentSuggestions,PrivacySandboxSettings4,AutofillServerCommunication,CalculateNativeWinOcclusion,OptimizationHints,AudioServiceOutOfProcess,IsolateOrigins,ImprovedCookieControls,LazyFrameLoading,GlobalMediaControls,DestroyProfileOnBrowserClose,MediaRouter,DialMediaRouteProvider,AcceptCHFrame,AutoExpandDetailsElement,CertificateTransparencyComponentUpdater,AvoidUnnecessaryBeforeUnloadCheckSync,Translate"
        ]
    };
    /// The light panda args to use.
    pub static ref LIGHTPANDA_ARGS: [&'static str; 2] = {
        let port = if DEFAULT_PORT.eq(&9223) {
            "--port=9223"
        } else if DEFAULT_PORT.eq(&9224) {
            "--port=9224"
        } else {
            "--port=9222"
        };

        [
            "--host=0.0.0.0",
            port,
        ]
    };

    pub static ref CLIENT: Client<hyper::client::HttpConnector> = {
        Client::new()
    };
    /// Return base target and replacement. Target port is the port for chrome.
    pub(crate) static ref TARGET_REPLACEMENT: (&'static [u8; 5], &'static[u8; 5]) = {
        if *DEFAULT_PORT == 9223 {
            let target_port = b":9223";
            let proxy_port = b":9222";

            (target_port, proxy_port)
        } else {
            // we need to allow dynamic ports instead of defaulting to standard and xfvb offport.
            let target_port = b":9224";
            let proxy_port = b":9223";

            (target_port, proxy_port)
        }
    };
}

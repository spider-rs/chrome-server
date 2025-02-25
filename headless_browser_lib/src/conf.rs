use std::collections::HashSet;
use std::sync::atomic::{AtomicBool, AtomicU64};

/// The performance arg count.
pub(crate) const PERF_ARGS: usize = 97;

#[cfg(any(test, feature = "testing"))]
lazy_static::lazy_static! {
    /// The chrome args to use test ( basic without anything used for testing ).
    pub static ref CHROME_ARGS_TEST: [&'static str; 6] = {
        let headless = std::env::args()
        .nth(6)
        .unwrap_or("true".into());

        let headless = if headless != "false" {
            match std::env::var("HEADLESS") {
                Ok(h) => {
                    if h == "false" {
                        ""
                    } else if h == "new" {
                        "--headless=new"
                    }else {
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

        let gpu = std::env::var("ENABLE_GPU").unwrap_or_default() == "true";

        let gpu_enabled = if gpu { "--enable-gpu" } else { "--disable-gpu" };
        let gpu_enabled_sandboxed = if gpu { "--enable-gpu-sandbox" } else { "--disable-gpu-sandbox" };

        [
            // *SPECIAL*
            "--remote-debugging-address=0.0.0.0",
            port,
            // *SPECIAL*
            headless,
            gpu_enabled,
            gpu_enabled_sandboxed,
            use_gl,
        ]
    };
}

lazy_static::lazy_static! {
    /// Is the instance healthy?
    pub static ref IS_HEALTHY: AtomicBool = AtomicBool::new(true);
    pub static ref CHROME_INSTANCES: tokio::sync::Mutex<HashSet<u32>> = tokio::sync::Mutex::new(HashSet::new());
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
    /// Is a brave instance?
    pub(crate) static ref BRAVE_INSTANCE: bool = {
        CHROME_PATH.ends_with("Brave Browser")
        || CHROME_PATH.ends_with("brave-browser")
    };
    /// Is a lightpanda instance?
    pub(crate) static ref LIGHT_PANDA: bool = {
        CHROME_PATH.ends_with("lightpanda-aarch64-macos")
        || CHROME_PATH.ends_with("lightpanda-x86_64-linux")
    };
    /// The chrome args to use.
    pub static ref CHROME_ARGS: [&'static str; PERF_ARGS] = {
        let headless = std::env::args()
        .nth(6)
        .unwrap_or("true".into());

        let headless = if headless != "false" {
            match std::env::var("HEADLESS") {
                Ok(h) => {
                    if h == "false" {
                        ""
                    } else if h == "new" {
                        "--headless=new"
                    }else {
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

        let gpu = std::env::var("ENABLE_GPU").unwrap_or_default() == "true";

        let gpu_enabled = if gpu { "--enable-gpu" } else { "--disable-gpu" };
        let gpu_enabled_sandboxed = if gpu { "--enable-gpu-sandbox" } else { "--disable-gpu-sandbox" };

        [
            // *SPECIAL*
            "--remote-debugging-address=0.0.0.0",
            port,
            // *SPECIAL*
            headless,
            gpu_enabled,
            gpu_enabled_sandboxed,
            use_gl,
            "--no-first-run",
            "--no-sandbox",
            "--disable-setuid-sandbox",
            "--no-zygote",
            "--hide-scrollbars",
            "--user-data-dir=~/.config/google-chrome",
            "--allow-running-insecure-content",
            "--autoplay-policy=user-gesture-required",
            "--ignore-certificate-errors",
            "--no-default-browser-check",
            "--disable-dev-shm-usage", // required or else container will crash not enough memory
            "--disable-threaded-scrolling",
            "--disable-cookie-encryption",
            "--disable-demo-mode",
            "--disable-dinosaur-easter-egg",
            "--disable-fetching-hints-at-navigation-start",
            "--disable-site-isolation-trials",
            "--disable-web-security",
            "--disable-threaded-animation",
            "--disable-sync",
            "--disable-print-preview",
            "--disable-search-engine-choice-screen",
            "--disable-partial-raster",
            "--disable-in-process-stack-traces",
            "--use-angle=swiftshader",
            "--disable-low-res-tiling",
            "--disable-speech-api",
            "--disable-oobe-chromevox-hint-timer-for-testing",
            "--disable-smooth-scrolling",
            "--disable-default-apps",
            "--disable-prompt-on-repost",
            "--disable-domain-reliability",
            "--enable-dom-distiller",
            "--enable-distillability-service",
            "--disable-component-update",
            "--disable-background-timer-throttling",
            "--disable-breakpad",
            "--disable-crash-reporter",
            "--disable-software-rasterizer",
            "--disable-asynchronous-spellchecking",
            "--disable-extensions",
            "--disable-html5-camera",
            "--noerrdialogs",
            "--disable-popup-blocking",
            "--disable-hang-monitor",
            "--disable-checker-imaging",
            "--enable-surface-synchronization",
            "--disable-image-animation-resync",
            "--disable-client-side-phishing-detection",
            "--disable-component-extensions-with-background-pages",
            "--run-all-compositor-stages-before-draw",
            "--disable-background-networking",
            "--disable-renderer-backgrounding",
            "--disable-field-trial-config",
            "--disable-back-forward-cache",
            "--disable-backgrounding-occluded-windows",
            "--log-level=3",
            "--enable-logging=stderr",
            "--font-render-hinting=none",
            "--block-new-web-contents",
            "--no-subproc-heap-profiling",
            "--no-pre-read-main-dll",
            "--disable-stack-profiler",
            "--disable-libassistant-logfile",
            "--crash-on-hang-threads",
            "--restore-last-session",
            "--ip-protection-proxy-opt-out",
            "--unsafely-disable-devtools-self-xss-warning",
            "--enable-features=PdfOopif,SharedArrayBuffer,NetworkService,NetworkServiceInProcess",
            "--metrics-recording-only",
            "--use-mock-keychain",
            "--force-color-profile=srgb",
            "--disable-infobars",
            "--mute-audio",
            "--disable-datasaver-prompt",
            "--no-service-autorun",
            "--password-store=basic",
            "--export-tagged-pdf",
            "--no-pings",
            "--rusty-png",
            "--disable-histogram-customizer",
            "--window-size=800,600",
            "--disable-vulkan-fallback-to-gl-for-testing",
            "--disable-vulkan-surface",
            "--disable-webrtc",
            "--disable-oopr-debug-crash-dump",
            "--disable-pnacl-crash-throttling",
            "--disable-renderer-accessibility",
            "--disable-blink-features=AutomationControlled",
            "--disable-ipc-flooding-protection", // we do not need to throttle navigation for https://github.com/spider-rs/spider/commit/9ff5bbd7a2656b8edb84b62843b72ae9d09af079#diff-75ce697faf0d37c3dff4a3a19e7524798b3cb5487f8f54beb5d04c4d48e34234R446.
            // --deterministic-mode 10-20% drop in perf
            // "--blink-settings=primaryHoverType=2,availableHoverTypes=2,primaryPointerType=4,availablePointerTypes=4",
            "--disable-features=PaintHolding,HttpsUpgrades,DeferRendererTasksAfterInput,LensOverlay,ThirdPartyStoragePartitioning,IsolateSandboxedIframes,ProcessPerSiteUpToMainFrameThreshold,site-per-process,WebUIJSErrorReportingExtended,DIPS,InterestFeedContentSuggestions,PrivacySandboxSettings4,AutofillServerCommunication,CalculateNativeWinOcclusion,OptimizationHints,AudioServiceOutOfProcess,IsolateOrigins,ImprovedCookieControls,LazyFrameLoading,GlobalMediaControls,DestroyProfileOnBrowserClose,MediaRouter,DialMediaRouteProvider,AcceptCHFrame,AutoExpandDetailsElement,CertificateTransparencyComponentUpdater,AvoidUnnecessaryBeforeUnloadCheckSync,Translate",
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
    /// The hostname of the machine to replace 127.0.0.1 when making request to /json/version on port 6000.
    pub(crate) static ref HOST_NAME: String = {
        let mut hostname = String::new();

        if let Ok(name) = std::env::var("HOSTNAME_OVERRIDE") {
            if !name.is_empty() {
                hostname = name;
            }
        }

        if hostname.is_empty() {
            if let Ok(name) = std::env::var("HOSTNAME") {
                if !name.is_empty() {
                    hostname = name;
                }
            }
        }

        hostname
    };
    /// The main endpoint for entry.
    pub(crate) static ref ENDPOINT_BASE: String = {
        format!("http://127.0.0.1:{}", *DEFAULT_PORT)
    };
    /// The main endpoint json/version.
    pub(crate) static ref ENDPOINT: String = {
        format!("http://127.0.0.1:{}/json/version", *DEFAULT_PORT)
    };
    /// The chrome launch path.
    pub static ref CHROME_PATH: String = {
        // cargo bench will always pass in the first arg
        let default_path = std::env::args().nth(1).unwrap_or_default();
        let trimmed_path = default_path.trim();

        // handle testing and default to OS
        if default_path.is_empty() || trimmed_path == "--nocapture" || trimmed_path == "--bench" {
            let chrome_path = match std::env::var("CHROME_PATH") {
                Ok(p) => p,
                _ => Default::default()
            };

            if chrome_path.is_empty() {
                get_default_chrome_bin().to_string()
            } else {
                chrome_path
            }
        } else {
            default_path
        }
    };
    /// The chrome address.
    pub(crate) static ref CHROME_ADDRESS: String = {
        let mut host_address = std::env::args().nth(2).unwrap_or("127.0.0.1".to_string()).to_string();

        if host_address.is_empty() {
            host_address = String::from("127.0.0.1").into()
        }

        host_address
    };
    pub(crate) static ref CACHEABLE: AtomicBool = {
        AtomicBool::new(true)
    };
    /// The last cache date period.
    pub(crate) static ref LAST_CACHE: AtomicU64 = {
        AtomicU64::new(0)
    };
    /// Debug the json version endpoint.
    pub(crate) static ref DEBUG_JSON: bool = std::env::var("DEBUG_JSON").unwrap_or_default() == "true";
    /// Test headless without args.
    pub(crate) static ref TEST_NO_ARGS: bool = std::env::var("TEST_NO_ARGS").unwrap_or_default() == "true";
}

/// Get the default chrome bin location per OS.
fn get_default_chrome_bin() -> &'static str {
    let brave = match std::env::var("BRAVE_ENABLED") {
        Ok(v) => v == "true",
        _ => false,
    };

    if cfg!(target_os = "windows") {
        if brave {
            "brave-browser.exe"
        } else {
            "chrome.exe"
        }
    } else if cfg!(target_os = "macos") {
        if brave {
            "/Applications/Brave Browser.app/Contents/MacOS/Brave Browser"
        } else {
            "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome"
        }
    } else if cfg!(target_os = "linux") {
        if brave {
            "brave-browser"
        } else {
            "chromium"
        }
    } else {
        if brave {
            "brave"
        } else {
            "chrome"
        }
    }
}

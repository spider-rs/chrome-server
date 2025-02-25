use chromiumoxide::browser::Browser;
use futures_util::stream::StreamExt;
use std::ops::Div;
use std::sync::Arc;
use std::{
    env,
    fs::{self, File, OpenOptions},
    io::{self, BufRead, Write},
    path::Path,
    time::{Duration, Instant},
};
use tokio::task::JoinSet;

const LOG_DIR: &str = "logs_concurrent";
const SAMPLE_COUNT: u32 = 10;

/// Run the benchmarks concurrently to the env BENCH_URL.
pub async fn run(log_file_name: &str) {
    ensure_log_directory_exists(LOG_DIR).expect("Failed to create log directory");
    let query = env::var("BENCH_URL").unwrap_or_else(|_| "http://spider.cloud".into());
    let q1 = query.clone();
    let mut total_duration = Duration::new(0, 0);
    let mut set = JoinSet::new();
    let query = Arc::new(query);
    let current_time = Instant::now();

    for i in 0..SAMPLE_COUNT {
        println!("Running sample {} of {}", i + 1, SAMPLE_COUNT);
        let query = query.clone();

        set.spawn(async move {
            let start_time = Instant::now();
            let result = navigate_extract_and_close(&query).await;
            let duration = start_time.elapsed();
            if let Err(e) = result {
                eprintln!("Error running test {}: {:?}", i + 1, e);
            } else {
                println!("Sample {} took: {:?}", i + 1, duration);
            }
            duration
        });
    }

    while let Some(res) = set.join_next().await {
        total_duration += res.unwrap_or_default();
    }

    let average_duration = total_duration.div(SAMPLE_COUNT);
    let total_time = current_time.elapsed();

    println!(
        "Finished average time: {:?} - total time: {:?}",
        average_duration, total_time
    );

    log_performance(total_time, average_duration, &q1, log_file_name)
        .expect("Failed to log performance");
}

/// Ensure the dir always exist.
fn ensure_log_directory_exists(dir: &str) -> io::Result<()> {
    if !Path::new(dir).exists() {
        fs::create_dir_all(dir)?;
    }
    Ok(())
}

/// Log the performance to file.
fn log_performance(
    total_duration: Duration,
    current_avg: Duration,
    query: &str,
    log_file_name: &str,
) -> io::Result<()> {
    let os_type = sys_info::os_type().unwrap_or_default();
    let cpu_count = sys_info::cpu_num().unwrap_or_default().to_string();
    let sanitized_os = os_type.replace(|c: char| !c.is_alphanumeric(), "_");

    // Construct the log file path with the machine information
    let log_file_name = format!("{}_v{}cpu_{}", sanitized_os, cpu_count, log_file_name);
    let log_file_path = format!("{}/{}", LOG_DIR, log_file_name);

    if let Ok(mut log_file) = OpenOptions::new()
        .read(true)
        .write(true)
        .append(true)
        .create(true)
        .open(log_file_path)
    {
        let chrome_args = if env::var("TEST_NO_ARGS").unwrap_or_default() == "true" {
            format!(
                "({})({:?})",
                headless_browser_lib::conf::CHROME_ARGS_TEST.len(),
                headless_browser_lib::conf::CHROME_ARGS_TEST.join(",")
            )
        } else {
            format!(
                "({})({:?})",
                headless_browser_lib::conf::CHROME_ARGS.len(),
                headless_browser_lib::conf::CHROME_ARGS.join(",")
            )
        };

        let chrome_path = headless_browser_lib::conf::CHROME_PATH
            .trim_end_matches('/')
            .to_string();
        let chrome_path = Path::new(&chrome_path)
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or_default();

        let last_benchmark = get_last_benchmark(&log_file)?;

        if let Some(last_avg) = last_benchmark {
            match current_avg.cmp(&last_avg) {
                std::cmp::Ordering::Greater => {
                    println!("Performance degraded. Previous average: {:?}", last_avg)
                }
                std::cmp::Ordering::Less => {
                    println!("Performance improved. Previous average: {:?}", last_avg)
                }
                std::cmp::Ordering::Equal => println!("Performance unchanged."),
            }
        }

        writeln!(
            log_file,
            "<{query}> - {SAMPLE_COUNT} SAMPLES\nCHROME_PATH: {}\nCHROME_ARGS: {}\nMACHINE: {}\nDATE: {}\nTotal Duration: {:?}\nAverage Duration: {:?}\n",
            chrome_path,
            chrome_args,
            format!("{}/v{}cpu", os_type, cpu_count),
            chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
            total_duration,
            current_avg
        )?;
    }
    Ok(())
}

/// Get the last benchmark results duration.
fn get_last_benchmark(log_file: &File) -> io::Result<Option<Duration>> {
    let mut lines = io::BufReader::new(log_file).lines();
    let mut last_line = None;
    while let Some(line) = lines.next() {
        last_line = Some(line?);
    }

    if let Some(last_line) = last_line {
        if let Some(duration_str) = last_line.split(',').next() {
            if let Some(duration_value) = duration_str.split(':').nth(1) {
                return Ok(Some(parse_duration(duration_value.trim())?));
            }
        }
    }
    Ok(None)
}

/// Parse the duration without the ms.
fn parse_duration(s: &str) -> io::Result<Duration> {
    if let Some(stripped) = s.strip_suffix("ms") {
        stripped
            .parse::<f64>()
            .map(|millis| Duration::from_millis(millis as u64))
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid duration format"))
    } else {
        Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Invalid duration format",
        ))
    }
}

/// Navigate, get the HTML, and close the page.
async fn navigate_extract_and_close(u: &str) -> Result<(), Box<dyn std::error::Error>> {
    let (browser, mut handler) =
        Browser::connect_with_config("http://127.0.0.1:6000/json/version", Default::default())
            .await?;

    let handle = tokio::task::spawn(async move {
        while let Some(h) = handler.next().await {
            if h.is_err() {
                break;
            }
        }
    });

    let page = browser.new_page(u).await?;
    page.wait_for_navigation().await?.content().await?;
    handle.abort(); // Abort the handle to drop the connection.

    Ok(())
}

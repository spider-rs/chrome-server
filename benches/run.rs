use super::utils::{ensure_log_directory_exists, get_last_benchmark, navigate_extract_and_close};
use std::ops::Div;
use std::{
    env,
    fs::OpenOptions,
    io::{self, Write},
    path::Path,
    time::{Duration, Instant},
};

const LOG_DIR: &str = "logs";

/// Run the benchmarks to the env BENCH_URL.
pub async fn run(log_file_name: &str, samples: u32) {
    ensure_log_directory_exists(LOG_DIR).expect("Failed to create log directory");
    let query = env::var("BENCH_URL").unwrap_or_else(|_| "http://spider.cloud".into());
    let mut total_duration = Duration::new(0, 0);
    let current_time = Instant::now();

    for i in 0..samples {
        println!("Running sample {} of {}", i + 1, samples);

        let start_time = Instant::now();
        let result = navigate_extract_and_close(&query).await;
        let duration = start_time.elapsed();

        if let Err(e) = result {
            eprintln!("Error running test {}: {:?}", i + 1, e);
        } else {
            println!("Sample {} took: {:?}", i + 1, duration);
        }

        total_duration += duration;
    }

    let average_duration = total_duration.div(samples);
    let total_time = current_time.elapsed();

    println!(
        "Finished average time: {:?} - total time: {:?}",
        average_duration, total_time
    );

    log_performance(total_time, average_duration, &query, log_file_name, samples)
        .expect("Failed to log performance");
}

/// Log the performance to file.
fn log_performance(
    total_duration: Duration,
    current_avg: Duration,
    query: &str,
    log_file_name: &str,
    samples: u32,
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
            "<{query}> - {samples} SAMPLES\nCHROME_PATH: {}\nCHROME_ARGS: {}\nMACHINE: {}\nDATE: {}\nTotal Duration: {:?}\nAverage Duration: {:?}\n",
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

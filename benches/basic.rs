mod run;
mod run_concurrent;
mod utils;

use std::env::set_var;

const LOG_FILE_NAME: &str = "benchmark_logs.txt";

#[tokio::main]
async fn main() {
    set_var("CHROME_INIT", "ignore"); // Ignore the auto start.
    set_var("HOSTNAME", "localhost");
    let samples = std::env::var("SAMPLES")
        .unwrap_or("10".into())
        .parse::<u32>()
        .unwrap_or(10);

    headless_browser_lib::fork(Some(*headless_browser_lib::conf::DEFAULT_PORT));
    let task = tokio::spawn(headless_browser_lib::run_main());
    tokio::time::sleep(std::time::Duration::from_millis(1000)).await; // Wait for the server to load.
    run::run(LOG_FILE_NAME, samples).await;
    run_concurrent::run(LOG_FILE_NAME, samples).await;
    headless_browser_lib::shutdown_instances().await;
    task.abort();
}

pub mod run;
use std::env::set_var;

const LOG_FILE_NAME: &str = "benchmark_logs.txt";

#[tokio::main]
async fn main() {
    set_var("CHROME_INIT", "ignore"); // Ignore the auto start.
    run::run(LOG_FILE_NAME).await
}

pub mod run;
use std::env::set_var;

const LOG_FILE_NAME: &str = "benchmark_noargs_logs.txt";

#[tokio::main]
async fn main() {
    set_var("CHROME_INIT", "ignore"); // Ignore the auto start.
    set_var("TEST_NO_ARGS", "true");
    run::run(LOG_FILE_NAME).await
}

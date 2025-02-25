use std::{env, time::{Duration, Instant}};
use chromiumoxide::browser::Browser;
use futures_util::stream::StreamExt;
use std::env::set_var;

/// Run the test 10 times manually and log times.
#[tokio::main]
async fn main() {
    set_var("CHROME_INIT", "ignore"); // ignore the auto start
    let query = env::var("BENCH_URL").unwrap_or("http://spider.cloud".into());
    let sample_count = 10;
    let mut total_duration = Duration::new(0, 0);

    headless_browser_lib::fork(Some(*headless_browser_lib::conf::DEFAULT_PORT)).await;
    tokio::spawn(headless_browser_lib::run_main());
    // wait for the server to load
    tokio::time::sleep(Duration::from_millis(1000)).await;

    for i in 0..sample_count {
        println!("Running sample {} of {}", i + 1, sample_count);

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

    let average_duration = total_duration / sample_count as u32;
    println!("Finished average time: {:?}", average_duration);
}

/// Test the basic CDP with all of the major chromium-based libs.
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

    browser.close().await?;
    let _ = handle.await;

    Ok(())
}
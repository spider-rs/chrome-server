use chromiumoxide::{browser::Browser, error::CdpError};
use futures_util::stream::StreamExt;
use std::{
    env::set_var,
    time::{Duration, Instant},
};

#[tokio::test]
/// Test the basic crawl with all of the major chromium based libs.
async fn basic() -> Result<(), Box<dyn std::error::Error>> {
    set_var("CHROME_INIT", "ignore"); // ignore the auto start
    tracing_subscriber::fmt::init();
    headless_browser_lib::fork(Some(*headless_browser_lib::conf::DEFAULT_PORT)).await;
    tokio::spawn(headless_browser_lib::run_main());
    tokio::time::sleep(Duration::from_millis(100)).await; // give a slight delay for now until we use a oneshot.

    let start = Instant::now();

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

    let navigate_page = async |u: &str| -> Result<String, CdpError> {
        let start = Instant::now();
        let page = browser.new_page(u).await?;
        println!("NewPage({u}): {:?}", start.elapsed());
        let start = Instant::now();
        let html = page.wait_for_navigation().await?.content().await?;
        println!("WaitForNavigationAndContent({u}): {:?}", start.elapsed());
        Ok::<String, CdpError>(html)
    };

    let (spider_html, example_html) = tokio::join!(
        navigate_page("https://spider.cloud"),
        navigate_page("https://example.com")
    );

    let elasped = start.elapsed();

    browser.close().await?;
    let _ = handle.await;

    let spider_html = spider_html?;
    let example_html = example_html?;

    assert_eq!(elasped.as_secs() <= 15, true);
    assert_eq!(spider_html.is_empty(), false);
    assert_eq!(spider_html.len() >= 1000, true);

    assert_eq!(example_html.is_empty(), false);
    assert_eq!(example_html.len() >= 400, true);

    println!("Time took: {:?}", elasped);

    Ok(())
}

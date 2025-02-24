use chromiumoxide::{browser::Browser, error::CdpError};
use futures_util::stream::StreamExt;
use std::time::Duration;

#[tokio::test]
async fn basic() -> Result<(), Box<dyn std::error::Error>> {
    tokio::spawn(headless_browser_lib::run_main());
    tokio::time::sleep(Duration::from_millis(200)).await; // give a slight delay for now until we use a oneshot.

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
        let page = browser.new_page(u).await?;
        let html = page.wait_for_navigation().await?.content().await?;
        Ok::<String, CdpError>(html)
    };

    let (spider_html, example_html) = tokio::join!(
        navigate_page("https://spider.cloud"),
        navigate_page("https://example.com")
    );

    browser.close().await?;
    let _ = handle.await;

    let spider_html = spider_html?;
    let example_html = example_html?;

    assert_eq!(spider_html.is_empty(), false);
    assert_eq!(spider_html.len() >= 1000, true);

    assert_eq!(example_html.is_empty(), false);
    assert_eq!(example_html.len() >= 400, true);

    Ok(())
}

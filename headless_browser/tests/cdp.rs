use chromiumoxide::browser::Browser;
use futures_util::stream::StreamExt;
use std::time::Duration;

#[tokio::test]
async fn basic() -> Result<(), Box<dyn std::error::Error>> {
    tokio::spawn(headless_browser_lib::run_main());
    tokio::time::sleep(Duration::from_millis(100)).await; // give a slight delay for now until we use a oneshot.

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

    let page = browser.new_page("https://spider.cloud").await?;
    let html = page.wait_for_navigation().await?.content().await?;

    browser.close().await?;
    let _ = handle.await;

    assert_eq!(html.is_empty(), false);
    assert_eq!(html.len() >= 1000, true);

    Ok(())
}

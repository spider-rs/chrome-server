use chromiumoxide::browser::Browser;
use futures_util::stream::StreamExt;
use std::{
    fs::{self, File},
    io::{self, BufRead},
    path::Path,
    time::Duration,
};

/// Ensure the dir always exist.
pub fn ensure_log_directory_exists(dir: &str) -> io::Result<()> {
    if !Path::new(dir).exists() {
        fs::create_dir_all(dir)?;
    }
    Ok(())
}

/// Get the last benchmark results duration.
pub fn get_last_benchmark(log_file: &File) -> io::Result<Option<Duration>> {
    let mut lines = io::BufReader::new(log_file).lines();
    let mut last_line = None;

    while let Some(line) = lines.next() {
        let next_line = line?;
        if !next_line.is_empty() {
            last_line = Some(next_line);
        }
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
pub fn parse_duration(s: &str) -> io::Result<Duration> {
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
pub async fn navigate_extract_and_close(u: &str) -> Result<(), Box<dyn std::error::Error>> {
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

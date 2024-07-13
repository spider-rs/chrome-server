use std;

/// build chrome instance to container
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let build_chrome = std::env::var("BUILD_CHROME").is_ok();

    // build chrome instance to system
    if build_chrome {
        // install chrome linux modules [todo: check existing]
        if cfg!(target_os = "linux") {
            use std::process::Command;
            let mut command = Command::new("apt");

            command
                .args(["get", "install", "-y", "google-chrome-stable"])
                .output()
                .expect("failed to install chrome");
        }
    }

    Ok(())
}

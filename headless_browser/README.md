
# headless-browser

The `headless-browser` application serves as the primary entry point for managing headless Chrome instances with integrated proxy and server capabilities. This solution is designed to scale efficiently, providing developers with a robust tool for web automation and testing.

## Installation

To install the `headless-browser`, ensure that you have Rust installed, and then run:

```sh
cargo install headless_browser
```

## Usage

Once installed, you can start the `headless-browser` application by executing the following command:

```sh
headless_browser
```

This command initializes the environment and sets up the headless Chrome instances with the configured proxy and server settings.

## Testing

The application is tested using the highly optimized concurrent control mechanisms provided by the [DevTools Protocol](https://chromedevtools.github.io/devtools-protocol/) through the [spider_chrome](https://github.com/spider-rs/spider/tree/main/spider_chrome) crate. This ensures robust testing capabilities for a variety of web interaction scenarios.

Run the following command to execute tests and capture outputs:

```sh
RUST_LOG=error,info cargo test --package headless_browser --test cdp -- --nocapture
```

Ensure you have the appropriate logging and test configurations set up to get detailed output from your test executions.

---

This documentation provides a comprehensive overview for users looking to get started with the `headless-browser` tool, detailing installation, execution, and testing processes.
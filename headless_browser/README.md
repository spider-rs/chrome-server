# headless-browser

The main entry to the program.

`cargo install headless_browser`

```sh
headless_browser
```

## Test

The test are ran using the highly optimized concurrent control to [DevTools Protocol](https://chromedevtools.github.io/devtools-protocol/) via [spider_chrome](https://github.com/spider-rs/spider/tree/main/spider_chrome).


```rust
RUST_LOG=error,info cargo test --package headless_browser --test cdp  -- --nocapture
```
# headless-browser-benches

Benches for the headless browsers. The benches may have multiple sections as command line args influence the performance.

## Tests

View the benchmarks for each browser below.

### Google Chrome

example with Google Chrome ( reliable and fast ).

```sh
HEADLESS=true CHROME_PATH="/Applications/Google Chrome.app/Contents/MacOS/Google Chrome" cargo test --package headless_browser --test cdp  -- --nocapture
# NewPage(https://example.com): 172.977417ms
# WaitForNavigationAndContent(https://example.com): 834.417µs
# NewPage(https://spider.cloud): 510.849ms
# WaitForNavigationAndContent(https://spider.cloud): 24.371167ms
# Time took: 943.520375ms
```

### Chrome Headless Shell

example with Google Chrome headless-shell: (4x faster).

```sh
 HEADLESS=true CHROME_PATH=./chrome-headless-shell/chromium_headless_shell-1155/chrome-mac/headless_shell cargo test --package headless_browser --test cdp  -- --nocapture
# NewPage(https://example.com): 51.755125ms
# WaitForNavigationAndContent(https://example.com): 840.084µs
# NewPage(https://spider.cloud): 221.127334ms
# WaitForNavigationAndContent(https://spider.cloud): 22.51475ms
# Time took: 270.031125ms
```

### Brave Browser

example with brave(10x-20x slower).

```sh
HEADLESS=true CHROME_PATH="/Applications/Brave Browser.app/Contents/MacOS/Brave Browser" cargo test --package headless_browser --test cdp  -- --nocapture
# tracing_subscriber - init success
# NewPage(https://example.com): 330.503792ms
# WaitForNavigationAndContent(https://example.com): 3.170792ms
# NewPage(https://spider.cloud): 572.666833ms
# WaitForNavigationAndContent(https://spider.cloud): 20.741291ms
# Time took: 5.972425416s
```

## Running

When you run the benches make sure to pass in the CHROME_PATH env.

```sh
HEADLESS=true CHROME_PATH=./chrome-headless-shell/chromium_headless_shell-1155/chrome-mac/headless_shell cargo bench
```

## Benchmarks

View the [benchmarks](./logs/) to see the runs with the machine used and history of the args for the performance.
The `Argless` has the headless instance launched with minimal args required to run.

### Mac

* [Darwin_v10cpu](./logs/Darwin_v10cpu_benchmark_logs.txt)
* [Darwin_v10cpu Argless](./logs/Darwin_v10cpu_benchmark_noargs_logs.txt)

### Linux

* [Linux_v4cpu](./logs/Linux_v4cpu_benchmark_logs.txt)
* [Linux_v4cpu Argless](./logs/Linux_v4cpu_benchmark_noargs_logs.txt)

# chrome

Chrome instance and panel to manage startup and shutdowns easily.

## Installation

`cargo install chrome_server`

## Usage

1. Can spawn and shutdown multiple chrome instances.
1. Get chrome ws connections and status.

The current instance binds chrome to 0.0.0.0 when starting via API.

Use the env variable `REMOTE_ADDRESS` to change the address of the chrome instance between physical or network.

The application will pass alp health checks when using port `6000` to get the status of the chrome container.

A side loaded application is required to run chrome on a load balancer, one of the main purposes of the control panel.

The default port is `9222` for chrome.

## Docker

You can use the docker images `a11ywatch/chrome` and `a11ywatch/chrome-xvfb`.

## Building without Docker

In order to build without docker set the `BUILD_CHROME` env var to true.

## Mac

If your running locally use the following to start the args with the first param `chrome_driver '/Applications/Google Chrome.app/Contents/MacOS/Google Chrome'`

## API

1. POST: `fork` to start a new chrome instance or use `fork/$port` with the port to startup the instance.
2. POST: `shutdown/$PID` to shutdown the instance. ex: `curl --location --request POST 'http://localhost:6000/shutdown/77057'`
3. POST: `/json/version` get the json info of the chrome instance to connect to web sockets.

### Curl Examples

`fork`

```sh
curl --location --request POST 'http://localhost:6000/fork'
```

`shutdown`

```sh
curl --location --request POST 'http://localhost:6000/shutdown'
```

`/json/version`

```sh
curl --location --request GET 'http://localhost:6000/json/version' \
--header 'Content-Type: application/json'
{
   "Browser": "HeadlessChrome/114.0.5735.133",
   "Protocol-Version": "1.3",
   "User-Agent": "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) HeadlessChrome/114.0.5735.133 Safari/537.36",
   "V8-Version": "11.4.183.23",
   "WebKit-Version": "537.36 (@fbfa2ce68d01b2201d8c667c2e73f648a61c4f4a)",
   "webSocketDebuggerUrl": "ws://127.0.0.1:9222/devtools/browser/74f18759-f4b3-4b1f-a68c-942570542f0e"
}
```

## Args

1. The first arg is the chrome application location example linux `'/opt/google/chrome/chrome'`.
2. The second arg is the chrome address `127.0.0.1`.
3. The third arg you can pass in `init` to auto start chrome on `9222`.

Example to start chrome (all params are optional):

```sh
chrome_driver '/Applications/Google Chrome.app/Contents/MacOS/Google Chrome' 127.0.0.1 init
# Chrome PID: 87659
# Chrome server at localhost:6000
# DevTools listening on ws://127.0.0.1:9222/devtools/browser/c789f9e0-7f65-495d-baee-243eb454ea15
```
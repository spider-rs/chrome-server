#!/bin/sh

set -e

REMOTE_ADDRESS="${REMOTE_ADDRESS:-127.0.0.1}";

exec chrome_driver & chromium-browser --headless --disable-software-rasterizer --no-sandbox --disable-gpu --hide-scrollbars --mute-audio --no-first-run \
    --remote-debugging-address=$REMOTE_ADDRESS --remote-debugging-port=9222 --max-wait-for-load=2500 --allow-running-insecure-content --autoplay-policy=user-gesture-required \
    --disable-default-apps --disable-storage-reset --disable-dev-shm-usage --disable-component-update --disable-sync --disable-background-networking \
    --disable-background-timer-throttling --disable-notifications --disable-accelerated-2d-canvas --disable-accelerated-video-decode --disable-extensions \
    --disable-popup-blocking --disable-renderer-backgrounding --disable-client-side-phishing-detection --disable-setuid-sandbox \
    --disable-features=TranslateUI BlinkGenPropertyTrees --ignore-certificate-errors --disable-http2 --disable-backgrounding-occluded-windows --no-default-browser-check \
    --metrics-recording-only --disable-component-extensions-with-background-pages --disable-threaded-animation \
    --disable-threaded-compositing --enable-background-thread-pool
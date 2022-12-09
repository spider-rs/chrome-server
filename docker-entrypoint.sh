#!/bin/sh

set -e

REMOTE_ADDRESS="${REMOTE_ADDRESS:-127.0.0.1}";

exec chrome_driver & chromium-browser --headless --no-sandbox --hide-scrollbars --mute-audio --no-first-run \
    --remote-debugging-address=$REMOTE_ADDRESS --remote-debugging-port=9222 --max-wait-for-load=2500 --allow-running-insecure-content \
    --autoplay-policy=user-gesture-required --enable-background-thread-pool --disable-gpu --disable-software-rasterizer \
    --disable-storage-reset --disable-dev-shm-usage \
    --disable-accelerated-video-decode --disable-setuid-sandbox \
    --disable-features=TranslateUI BlinkGenPropertyTrees --ignore-certificate-errors --disable-component-extensions-with-background-pages
    
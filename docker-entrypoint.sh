#!/bin/sh

set -e

REMOTE_ADDRESS="${REMOTE_ADDRESS:-127.0.0.1}";

exec chrome_driver & chromium-browser --headless --no-sandbox --hide-scrollbars --no-first-run --disable-sync --enable-automation --disable-popup-blocking --disable-prompt-on-repost \
    --remote-debugging-address=$REMOTE_ADDRESS --remote-debugging-port=9222 --max-wait-for-load=2500 --allow-running-insecure-content --metrics-recording-only --allow-pre-commit-input \
    --autoplay-policy=user-gesture-required --enable-background-thread-pool --disable-gpu --disable-software-rasterizer --disable-ipc-flooding-protection \
    --disable-storage-reset --disable-background-timer-throttling --disable-dev-shm-usage --disable-backgrounding-occluded-windows --disable-partial-raster --disable-breakpad --disable-default-apps \
    --disable-accelerated-video-decode --disable-setuid-sandbox --disable-domain-reliability --no-pings --disable-hang-monitor --disable-extensions \
    --disable-features=BackForwardCache,AcceptCHFrame,AvoidUnnecessaryBeforeUnloadCheckSync,Translate,InterestFeedContentSuggestions,BlinkGenPropertyTrees --ignore-certificate-errors --disable-component-extensions-with-background-pages
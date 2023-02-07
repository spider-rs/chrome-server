#!/bin/sh

set -e

REMOTE_ADDRESS="${REMOTE_ADDRESS:-127.0.0.1}";
LOG_ENABLED=${LOG_ENABLED:-0};

exec chrome_driver & chromium-browser --headless=new --no-sandbox --hide-scrollbars --no-first-run --disable-sync --enable-automation --disable-popup-blocking --disable-prompt-on-repost \
    --remote-debugging-address=$REMOTE_ADDRESS --remote-debugging-port=9222 --max-wait-for-load=2500 --allow-running-insecure-content --metrics-recording-only --allow-pre-commit-input --disable-background-networking \
    --autoplay-policy=user-gesture-required --enable-background-thread-pool --disable-gpu --disable-software-rasterizer --disable-ipc-flooding-protection --mute-audio --disable-client-side-phishing-detection \
    --disable-storage-reset --disable-background-timer-throttling --disable-dev-shm-usage --disable-backgrounding-occluded-windows --disable-partial-raster --disable-breakpad --disable-default-apps --enable-logging=$LOG_ENABLED \
    --disable-accelerated-video-decode --disable-setuid-sandbox --disable-domain-reliability --no-pings --disable-hang-monitor --disable-extensions --user-data-dir=/usr/src/app --force-fieldtrials=*BackgroundTracing/default/ \
    --disable-features=BackForwardCache,AcceptCHFrame,AvoidUnnecessaryBeforeUnloadCheckSync,Translate,InterestFeedContentSuggestions,BlinkGenPropertyTrees,MediaRouter,OptimizationHints --ignore-certificate-errors --disable-component-extensions-with-background-pages
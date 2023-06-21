#!/bin/sh

set -e

REMOTE_ADDRESS="${REMOTE_ADDRESS:-127.0.0.1}";
LOG_ENABLED=${LOG_ENABLED:-0};

exec chrome_driver & chromium-browser --headless --no-sandbox --hide-scrollbars --no-first-run --disable-sync --enable-automation --disable-popup-blocking --disable-prompt-on-repost \
    --remote-debugging-address=$REMOTE_ADDRESS --remote-debugging-port=9222 --metrics-recording-only --allow-pre-commit-input \
    --autoplay-policy=user-gesture-required --disable-gpu --disable-gpu-sandbox  --mute-audio --disable-client-side-phishing-detection --ignore-certificate-errors --disable-component-extensions-with-background-pages \
    --disable-software-rasterizer --disable-dev-shm-usage --disable-backgrounding-occluded-windows --disable-breakpad --disable-default-apps --enable-logging=0 \
    --disable-accelerated-video-decode --disable-setuid-sandbox --disable-domain-reliability --no-pings --disable-hang-monitor --disable-extensions \
    --disable-features=BackForwardCache,AcceptCHFrame,AvoidUnnecessaryBeforeUnloadCheckSync,Translate,InterestFeedContentSuggestions,BlinkGenPropertyTrees,MediaRouter,OptimizationHints
#!/bin/sh

set -e

REMOTE_ADDRESS="${REMOTE_ADDRESS:-127.0.0.1}";
LAUNCH="${LAUNCH:-init}";

exec chrome_driver chromium-browser $REMOTE_ADDRESS $LAUNCH
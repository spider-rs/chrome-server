#!/bin/sh

set -e

REMOTE_ADDRESS="${REMOTE_ADDRESS:-127.0.0.1}";
LAUNCH="${LAUNCH:-init}";
DEFAULT_PORT="${DEFAULT_PORT:-9222}";
DEFAULT_PORT_SERVER="${DEFAULT_PORT_SERVER:-6000}";

echo "Starting Xvfb"

Xvfb :0 -screen 0 1024x768x16 -nolisten tcp &
sleep 1

exec chrome_driver chromium-browser $REMOTE_ADDRESS $LAUNCH $DEFAULT_PORT $DEFAULT_PORT_SERVER "false"
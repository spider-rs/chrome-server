#!/bin/sh

set -e

REMOTE_ADDRESS="${REMOTE_ADDRESS:-127.0.0.1}";
LAUNCH="${LAUNCH:-init}";
DEFAULT_PORT="${DEFAULT_PORT:-9223}";
DEFAULT_PORT_SERVER="${DEFAULT_PORT_SERVER:-6000}";
# HEADLESS="new"

exec chrome_server chromium-browser $REMOTE_ADDRESS $LAUNCH $DEFAULT_PORT $DEFAULT_PORT_SERVER "true"
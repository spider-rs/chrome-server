#!/bin/bash

# Run the chrome build script
# you also need to install some other deps like go and etc. Follow https://github.com/chromedp/docker-headless-shell

# one liner to install on AWS arm
# sudo yum update -y && sudo yum install -y wget tar && wget https://dl.google.com/go/go1.23.5.linux-arm64.tar.gz && sudo rm -rf /usr/local/go && sudo tar -C /usr/local -xzf go1.23.5.linux-arm64.tar.gz && rm go1.23.5.linux-arm64.tar.gz && echo 'export PATH=$PATH:/usr/local/go/bin' >> ~/.bashrc && source ~/.bashrc && go version

# Get the verhist module
go install github.com/chromedp/verhist/cmd/verhist@latest

# Verify installation
if go list -m github.com/chromedp/verhist@latest &> /dev/null; then
    echo "verhist module installed successfully."
else
    echo "Failed to install verhist module."
fi

OUT_DIR="out"

[ -d "$OUT_DIR" ] || (echo "Creating directory '$OUT_DIR'..." && mkdir "$OUT_DIR")

./build-base.sh
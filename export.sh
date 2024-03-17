#!/usr/bin/env bash

mkdir -p /mnt/export
echo "Copying rgpio to /mnt/export..."
cp -f /usr/src/target/arm-unknown-linux-gnueabihf/release/rgpio /mnt/export/
echo "Copied rpgio to /mnt/export."

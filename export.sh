#!/usr/bin/env bash

mkdir -p /mnt/export
ls -l /usr/src/target
ls -l /mnt/export
cp -rf /usr/src/target /mnt/export
echo "Copied target to /mnt/export"

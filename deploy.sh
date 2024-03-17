#!/usr/bin/env bash

set -e

HOST=${1:-gdoor}
WORK_DIR=~/rgpio
DEPLOY_DIR=/srv/http/rgpio

docker buildx build . -t rgpio_build
docker run -v ./export:/mnt/export rgpio_build

#rsync -avF --delete . "$HOST:$WORK_DIR"
scp ./export/target/arm-unknown-linux-gnueabihf/release/rgpio "$HOST:$WORK_DIR/"
scp ./rgpio.service "$HOST:$WORK_DIR/"

ssh "$HOST" WORK_DIR=$WORK_DIR DEPLOY_DIR=$DEPLOY_DIR 'bash -s' <<'EOF'
    cd $WORK_DIR &&
    mkdir -p $DEPLOY_DIR &&
    cp -f rgpio $DEPLOY_DIR/ &&
    sudo cp -f rgpio.service /usr/lib/systemd/system/ &&
    sudo systemctl daemon-reload &&
    sudo systemctl stop rgpio &&
    sudo systemctl --now enable rgpio
EOF

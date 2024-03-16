#!/usr/bin/env bash

HOST=${1:-gdoor}
WORK_DIR=~/rgpio
DEPLOY_DIR=/srv/http/rgpio

rsync -avF --delete . "$HOST:$WORK_DIR"

ssh "$HOST" WORK_DIR=$WORK_DIR DEPLOY_DIR=$DEPLOY_DIR 'bash -s' <<'EOF'
    cd $WORK_DIR &&
    cargo build -r &&
    mkdir -p $DEPLOY_DIR &&
    cp -f target/release/rgpio $DEPLOY_DIR/ &&
    sudo cp -f rgpio.service /usr/lib/systemd/system/ &&
    sudo systemctl daemon-reload &&
    sudo systemctl --now enable rgpio
EOF

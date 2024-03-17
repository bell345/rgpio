#!/usr/bin/env bash

set -e

HOST=${1:-gdoor}
#WORK_DIR=~/rgpio
DEPLOY_DIR=/srv/http/rgpio

docker buildx build . -t rgpio_build
docker run -v ./export:/mnt/export rgpio_build

#rsync -avF --delete . "$HOST:$WORK_DIR"
scp ./export/rgpio ./rgpio.service ./rgpio.toml "$HOST:$DEPLOY_DIR/"

ssh "$HOST" DEPLOY_DIR=$DEPLOY_DIR 'bash -s' <<'EOF'
    cd $DEPLOY_DIR &&
    sudo cp -f rgpio.service /usr/lib/systemd/system/ &&
    echo 'Reloading systemd...' &&
    sudo systemctl daemon-reload &&
    echo 'Enabling rgpio...' &&
    sudo systemctl enable rgpio &&
    echo 'Restarting rgpio...' &&
    sudo systemctl restart rgpio
EOF

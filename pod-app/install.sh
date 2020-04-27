#!/usr/bin/env bash
set -ex

INSTALL_DIR="$HOME/.local/share/pod-app"

cargo install --path . --force

rm --recursive $INSTALL_DIR
mkdir -p $INSTALL_DIR

cp -f ../pod-enclave/pod_enclave.signed.so $INSTALL_DIR


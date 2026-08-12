#!/usr/bin/env bash
set -euo pipefail
export DEBIAN_FRONTEND=noninteractive
apt-get update -qq
apt-get install -y -qq build-essential pkg-config libssl-dev cmake 2>&1 | tail -5
echo "---"
cc --version | head -1
ld --version | head -1

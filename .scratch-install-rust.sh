#!/usr/bin/env bash
set -euo pipefail
export RUSTUP_HOME="${RUSTUP_HOME:-$HOME/.rustup}"
export CARGO_HOME="${CARGO_HOME:-$HOME/.cargo}"
chmod +x /tmp/rustup-init
/tmp/rustup-init -y --profile minimal --default-toolchain stable --no-modify-path
"$CARGO_HOME/bin/rustup" component add clippy rustfmt
"$CARGO_HOME/bin/rustc" --version
"$CARGO_HOME/bin/cargo" --version

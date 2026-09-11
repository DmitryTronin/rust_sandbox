#!/usr/bin/env bash

set -euo pipefail

project_dir="$(pwd)"
cargo_env="$HOME/.cargo/env"

install_login_hook() {
  local profile marker hook
  marker="# rust_sandbox Cargo environment"
  hook="[ -f \"$cargo_env\" ] && . \"$cargo_env\" $marker"

  for profile in "$HOME/.bash_profile" "$HOME/.bash_login" "$HOME/.profile"; do
    if [ -f "$profile" ]; then
      break
    fi
  done
  if [ ! -f "$profile" ]; then
    profile="$HOME/.profile"
    touch "$profile"
  fi

  for profile in "$profile" "$HOME/.bashrc"; do
    touch "$profile"
    if ! grep -Fq "$marker" "$profile"; then
      printf '\n%s\n%s\n' "$marker" "$hook" >> "$profile"
    fi
  done
}

ensure_rust() {
  if [ ! -x "$HOME/.cargo/bin/cargo" ]; then
    echo "Installing the Rust toolchain..."
    curl --fail --location --proto '=https' --tlsv1.2 --proxy "${HTTPS_PROXY:?HTTPS_PROXY must be set}" https://sh.rustup.rs \
      | sh -s -- -y --profile minimal
  fi

  # shellcheck disable=SC1090
  . "$cargo_env"
  install_login_hook
  cargo --version
}

start_service() {
  if curl --fail --silent http://127.0.0.1:7878/health >/dev/null 2>&1; then
    echo "Analyzer service is already running on port 7878."
    return
  fi

  echo "Starting the analyzer service on port 7878..."
  nohup cargo run > /tmp/rust_sandbox-service.log 2>&1 &
}

healthcheck() {
  echo "Waiting for the analyzer service to become ready..."
  while true; do
    if curl --fail --silent http://127.0.0.1:7878/health | grep -Fq '"status":"ok"' \
      && curl --fail --silent http://127.0.0.1:7878/ui | grep -Fq 'Clipboard Hidden Character Analyzer' \
      && printf 'hello\302\240world\342\200\213' \
        | curl --fail --silent --request POST http://127.0.0.1:7878/analyze-clipboard --data-binary @- \
        | grep -Fq '"code_point":"U+00A0"'; then
      echo "Analyzer service passed healthcheck."
      return 0
    fi

    echo "Service is not ready yet; recent log output:"
    tail -n 20 /tmp/rust_sandbox-service.log 2>/dev/null || true
    sleep 2
  done
}

cd "$project_dir"
ensure_rust

echo "Building and testing rust_sandbox to warm Cargo caches..."
cargo build
cargo test

start_service

if [ "${AIR_STARTUP_MODE:-}" = "warmup" ]; then
  healthcheck
fi

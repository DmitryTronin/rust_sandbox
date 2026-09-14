#!/usr/bin/env bash
set -euo pipefail

workspace_dir="$(git rev-parse --show-toplevel)"
target_triple="x86_64-unknown-linux-musl"
host_triple="$(rustc -vV | sed -n 's/^host: //p')"
rust_lld="$(rustc --print sysroot)/lib/rustlib/${host_triple}/bin/rust-lld"

if [ ! -x "$rust_lld" ]; then
  echo "Rust linker was not found at $rust_lld" >&2
  exit 1
fi

mkdir -p "$HOME/.cargo"
cat >"$HOME/.cargo/config.toml" <<EOF
[build]
target = "${target_triple}"

[target.${target_triple}]
linker = "${rust_lld}"
EOF

env_file="$HOME/.air-rust-sandbox-env"
printf 'export CARGO_BUILD_TARGET=%q\n' "$target_triple" >"$env_file"
source_line="[ -f '$env_file' ] && . '$env_file' # air-rust-sandbox"
profile_file=""
for candidate in "$HOME/.bash_profile" "$HOME/.bash_login" "$HOME/.profile"; do
  if [ -f "$candidate" ]; then
    profile_file="$candidate"
    break
  fi
done
if [ -z "$profile_file" ]; then
  profile_file="$HOME/.profile"
  touch "$profile_file"
fi
for shell_file in "$profile_file" "$HOME/.bashrc"; do
  touch "$shell_file"
  grep -Fqx "$source_line" "$shell_file" || printf '%s\n' "$source_line" >>"$shell_file"
done

echo "Installing the Rust standard library for $target_triple"
rustup target add "$target_triple"

cd "$workspace_dir"
echo "Fetching and building Rust dependencies"
cargo fetch
cargo test --target "$target_triple"
cargo build --target "$target_triple"

healthcheck() {
  echo "Waiting for the analyzer service on port 7878"
  until curl -fsS -H 'Host: warmup.local' http://127.0.0.1:7878/health | grep -q '"status":"ok"'; do
    echo "Analyzer service is not ready yet"
    sleep 1
  done

  echo "Checking analyzer API behavior"
  curl -fsS -H 'Host: warmup.local' -X POST http://127.0.0.1:7878/analyze-clipboard \
    --data-binary $'hello\u00a0world\u200b' | grep -q '"invisible_count":2'
  echo "Analyzer service is ready"
}

echo "Starting the analyzer service"
nohup "$workspace_dir/target/$target_triple/debug/temp-1" >"/tmp/rust-sandbox.log" 2>&1 &

if [ "${AIR_STARTUP_MODE:-}" = "warmup" ]; then
  healthcheck
fi

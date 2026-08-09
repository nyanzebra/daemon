#!/usr/bin/env bash
set -euo pipefail

# Builds the Docker image and runs this crate's test suite plus both
# examples inside a real systemd instance, in a disposable container.
#
# Needs --privileged and a cgroup mount, since systemd inside the container
# has to manage its own cgroup hierarchy — this is the standard
# "systemd-in-Docker" pattern, nothing specific to this crate. Only run this
# against a Docker daemon you trust: --privileged effectively disables
# container isolation for the duration of the run.

cd "$(dirname "$0")/.."

IMAGE=daemon-test
CONTAINER=daemon-test-run

docker build -t "$IMAGE" -f docker/Dockerfile .

docker rm -f "$CONTAINER" >/dev/null 2>&1 || true
cleanup() { docker rm -f "$CONTAINER" >/dev/null 2>&1 || true; }
trap cleanup EXIT

docker run -d \
    --name "$CONTAINER" \
    --privileged \
    --cgroupns=host \
    -v /sys/fs/cgroup:/sys/fs/cgroup:rw \
    --tmpfs /run \
    --tmpfs /run/lock \
    "$IMAGE"

echo "waiting for systemd to come up..."
for _ in $(seq 1 30); do
    if docker exec "$CONTAINER" systemctl is-system-running --wait >/dev/null 2>&1; then
        break
    fi
    sleep 1
done

echo "=== install_daemon example ==="
docker exec -u root "$CONTAINER" bash -c '
    set -e
    export RUST_LOG=debug
    cargo build --release --example exampled
    cargo run --example install_daemon -- --option install
    cargo run --example install_daemon -- --option start
    systemctl status exampled --no-pager
    cargo run --example install_daemon -- --option  stop
    cargo run --example install_daemon -- --option  uninstall
'

echo "=== install_daemon_from_toml example ==="
docker exec -u root "$CONTAINER" bash -c '
    set -e
    export RUST_LOG=debug
    cargo build --release --example exampled
    cargo run --example install_daemon_from_toml -- --option install
    cargo run --example install_daemon_from_toml -- --option start
    systemctl status exampled --no-pager
    cargo run --example install_daemon_from_toml -- --option stop
    cargo run --example install_daemon_from_toml -- --option uninstall
'

echo "done."

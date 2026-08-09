# Runs this crate's tests and examples inside a real systemd instance.
#
# Testing "install a user, group, systemd unit, and files under /etc,
# /var/lib, /var/log" for real needs both root and a live systemd to talk
# to. This image provides both in a disposable container instead of doing
# it on your host — see docker/run-tests.sh for how to run it.

FROM debian:bookworm

# `systemd`/`systemd-sysv` give this container a real PID 1 and a working
# `systemctl` to test against. `useradd`/`groupadd`/etc. come from the base
# image's shadow-utils package, already present.
RUN apt-get update && apt-get install -y --no-install-recommends \
    systemd \
    systemd-sysv \
    ca-certificates \
    curl \
    build-essential \
    && rm -rf /var/lib/apt/lists/*

ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:$PATH
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
    | sh -s -- -y --default-toolchain stable --profile minimal

WORKDIR /workspace
COPY . .

# Build once at image-build time so `docker run` doesn't rebuild on every
# invocation. Building doesn't touch the running system — only actually
# running the tests/examples does, which happens later, after systemd is up.
RUN cargo build --tests --release --examples

STOPSIGNAL SIGRTMIN+3
ENTRYPOINT ["/lib/systemd/systemd"]

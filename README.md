# daemon

A small library for installing and uninstalling a Linux daemon: creates the
system user/group, standard-location directories, the daemon binary, and a
systemd unit — and cleanly reverses all of it on uninstall.

## What `install()` does (requires root)

1. Creates any declared groups — skips ones that already exist.
2. Creates any declared users — skips ones that already exist — and adds
   them to their groups.
3. Creates any declared directories under FHS-standard roots (see below)
   with the given owner, group, and permissions.
4. Copies the daemon binary into place and makes it executable.
5. Writes the systemd unit file.
6. Writes a "bootstrap" config file, but only if one doesn't already exist —
   it will never overwrite existing configuration.
7. Reloads systemd and prints a `help` message.

`uninstall()` reverses all of it in a safe order — stopping and disabling
the service before removing the users/groups it depends on — and will never
delete a bootstrap file it didn't create in the first place.

Every mutating operation checks `geteuid() == 0` first and returns
`Error::NotRoot` if the process isn't running as root.

## Directories are FHS-anchored, not arbitrary paths

`Directory` and `Bootstrap` take a `DaemonPath` (`Configuration`, `State`,
`Cache`, `Logs`, `Runtime`) rather than a raw path string:

```rust
Directory::new(
    DaemonPath::State(RelativePath::new("mydaemon")?),
    Permissions::from_mode(0o750),
    "mydaemon", // group
    "mydaemon", // user
),
// -> /var/lib/mydaemon
```

This is deliberate. Since everything here runs as root, there's no config
field anywhere that can smuggle in a `..` traversal or an unexpected
absolute path — the only reachable destinations are:

| `DaemonPath`   | Resolves under  |
|-------------------|-----------------|
| `Configuration`   | `/etc`          |
| `State`           | `/var/lib`      |
| `Cache`           | `/var/cache`    |
| `Logs`            | `/var/log`      |
| `Runtime`         | `/run`          |


## Running the tests and examples

Both the test suite and the examples create a real system user/group and
write real files under `/etc`, `/var/lib`, `/var/log`, and
`/etc/systemd/system` — and the systemd-related tests/examples need an
actual running `systemd` to talk to, not just root. **Don't run either
directly on your host.**

The easiest way is the provided Docker harness, which builds a
systemd-enabled image and runs the full test suite plus both examples
against it, in a disposable container:

```bash
./docker/run-tests.sh
```

This needs `--privileged` and a cgroup mount (see
[`docker/run-tests.sh`](docker/run-tests.sh) and
[`docker/Dockerfile`](docker/Dockerfile)) — systemd inside the container
needs to manage its own cgroup hierarchy, the same requirement any
"systemd-in-Docker" setup has. Only run this against a Docker daemon you
trust, since `--privileged` disables container isolation for the run.

If you'd rather run things manually against the same image (e.g. to poke
around interactively), start it the same way the script does, then `docker
exec` into it:

```bash
docker build -t daemon-test -f docker/Dockerfile .
docker run -d --name daemon-test-run --privileged --cgroupns=host \
    -v /sys/fs/cgroup:/sys/fs/cgroup:rw --tmpfs /run --tmpfs /run/lock \
    daemon-test
docker exec -it daemon-test-run bash
# exampled is the program to install as daemon, build this first.
cargo build --release --example exampled
cargo run --example install_daemon -- --option install
```

## Examples

[`examples/install_daemon.rs`](examples/install_daemon.rs) builds a `Daemon`
config directly in Rust; [`examples/install_daemon_from_toml.rs`](examples/install_daemon_from_toml.rs)
describes the same daemon in [`examples/exampled.toml`](examples/exampled.toml)
and loads it via `Deserialize` instead. Run either through the Docker
harness above, or directly as root against a real systemd:

```bash
cargo build --example install_daemon
sudo ./target/debug/examples/install_daemon --option install
sudo ./target/debug/examples/install_daemon --option start
sudo ./target/debug/examples/install_daemon --option stop
sudo ./target/debug/examples/install_daemon --option uninstall
```

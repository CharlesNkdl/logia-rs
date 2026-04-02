# logia-rs


This is a little side project to improve in rust logic and tui implementation.
It's a simple log agglomerator, much needed in my little enterprise where monitoring and log agglomeration is still not necessarily in place. `logia-rs` gives you a terminal interface to browse and follow logs across a Unix system.

---

## Features

- Scans configured paths recursively and lists all readable `.log` files
- Navigate the filesystem directly from the TUI, go to any arbitrary path with `g`
- Shallow scan a directory (up to 3 levels deep) with `s`
- Connect to remote servers over SSH and browse their logs the same way
- Static log reading with manual refresh (`r`)
- Scroll through logs with arrow keys, `j/k`, `PgUp/PgDn`, `Home/End`
- Fuzzy search across file names and inside log content
- Export filtered log snapshots to a local file
- Three color themes, cycle with `t`
- Add SSH servers directly from the TUI

there is a laravel focus on parsing and color since we mostly work with laravel

---

## Usage

```bash
# Run in dev mode
make run

# Build a release binary
make release
./target/release/logia-rs
```

## Install

If you have the Rust toolchain:

```bash
make install
```
if you have the binary

```bash
tar -xzf logia-rs-*-your-platform.tar.gz
mv logia-rs ~/.local/bin/logia
chmod +x ~/.local/bin/logia
```

Make sure `~/.local/bin` is in your `PATH`:

```bash
export PATH="$HOME/.local/bin:$PATH"
```

## Configuration

Config lives at `~/.config/logia/config.toml`. You can define custom scan paths and SSH servers there.

```toml
local_paths = ["/var/log", "/var/www/html/storage/logs"]

[[servers]]
name = "prod-1"
host = "1.2.3.4"
port = 22
username = "deploy"
key_path = "~/.ssh/id_rsa"
```

## Development

```bash
make format   # cargo fmt
make lint     # cargo clippy
make test     # cargo test
make clean    # fmt + clippy
make docs     # cargo doc
make all      # format -> lint -> test -> run
```

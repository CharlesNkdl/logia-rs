# logia-rs


This is a little side project to improve in rust logic and tui implementation.
It's a simple log agglomerator, much needed in my little enterprise where monitoring and log agglomeration is still not necessarily in place. `logia-rs` gives you a terminal interface to browse and follow logs across a Unix system.

---

## Features

- Scans `/var/log/` recursively and lists all readable log files
- Navigate sources with the keyboard, open any log with Enter
- Real-time updates — new lines are picked up automatically on each tick

---

## Usage

```bash
# Run in dev mode
make run

# Build a release binary
make release
./target/release/logia-rs
```

## Development

```bash
make format   # cargo fmt
make lint     # cargo clippy
make test     # cargo test
make clean    # fmt + clippy
make docs     # cargo doc
make all      # format → lint → test → run
```

## To be done 
Implementation of config file or a superior parsing, to handle project specific logs locations.
main interest is right now -> laravel logs parsing
on top of that, i would need more test to use it in conjunction to ssh, to not having to download it to each server
- more test
- fuzzy finder in log file to help finding info
- ????
- probably more
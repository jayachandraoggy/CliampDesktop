# Cliamp Rust Desktop

A Rust desktop GUI for [cliamp](https://www.cliamp.stream/) that uses **cliamp** as the playback engine (headless daemon + IPC V2) and surfaces:

- **Cliamp Radio** — free channels from [radio.cliamp.stream](https://radio.cliamp.stream/)
- **Radio Browser** — browse and search ~58k stations through cliamp’s `Radio` provider (same as the TUI)

## Prerequisites

### Install from release (recommended)

Download `cliamprustdesktop_<version>_amd64.deb` from [GitHub Releases](https://github.com/jayachandraoggy/CliampDesktop/releases):

```sh
sudo dpkg -i cliamprustdesktop_*_amd64.deb
sudo apt-get install -f
```

The `.deb` installs **Cliamp Desktop** and **cliamp** (`/usr/bin`) — no separate cliamp install step.

### Build from source

- **[cliamp](https://www.cliamp.stream/)** on your `PATH`  
  `curl -fsSL https://cliamp.stream/install.sh | sh`  
  Or run `./scripts/fetch-cliamp.sh` and put `packaging/cliamp` on your `PATH` as `cliamp`.
- **[ffmpeg](https://ffmpeg.org/)** (recommended for AAC, Opus, HLS)
- Linux (Unix socket IPC at `~/.config/cliamp/cliamp.sock`)

## Releases

Push a version tag matching `Cargo.toml` (e.g. `v0.1.0`) to trigger the release workflow: `.deb`, tarball (app + cliamp), and checksums are published automatically.

## Build and run

```sh
cargo run --release
```

On first launch the app connects to an existing cliamp daemon or starts `cliamp --daemon`. Quitting the app leaves the daemon running.

## Attribution

Playback, decoding, and Radio Browser access are provided by [cliamp](https://github.com/bjarneo/cliamp).  
Cliamp Radio metadata is served from `radio.cliamp.stream`.

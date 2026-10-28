# Cliamp Rust Desktop

A Rust desktop GUI for [cliamp](https://www.cliamp.stream/) that uses **cliamp** as the playback engine (headless daemon + IPC V2) and surfaces:

- **Cliamp Radio** — free channels from [radio.cliamp.stream](https://radio.cliamp.stream/)
- **Radio Browser** — browse and search ~58k stations through cliamp’s `Radio` provider (same as the TUI)

## Prerequisites

- **[cliamp](https://www.cliamp.stream/)** on your `PATH`  
  `curl -fsSL https://cliamp.stream/install.sh | sh`
- **[ffmpeg](https://ffmpeg.org/)** (recommended for AAC, Opus, HLS)
- Linux (Unix socket IPC at `~/.config/cliamp/cliamp.sock`)

## Build and run

```sh
cargo run --release
```

On first launch the app connects to an existing cliamp daemon or starts `cliamp --daemon`. Quitting the app leaves the daemon running.

## Attribution

Playback, decoding, and Radio Browser access are provided by [cliamp](https://github.com/bjarneo/cliamp).  
Cliamp Radio metadata is served from `radio.cliamp.stream`.

Hertz release bundle (Linux x86_64)

This archive contains:
  hertz   — GUI application
  cliamp  — playback engine (daemon + IPC)

Quick install (user PATH):
  mkdir -p ~/.local/bin
  cp hertz cliamp ~/.local/bin/
  Ensure ~/.local/bin is on your PATH.

Recommended: install the .deb from GitHub Releases for menu integration
and automatic dependency hints:
  sudo dpkg -i hertz_<version>_amd64.deb
  sudo apt-get install -f

ffmpeg is recommended for AAC, Opus, and HLS streams.

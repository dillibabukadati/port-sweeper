Port Sweeper (psweep)

Find and kill processes by port. A lightning-fast CLI and GUI for when you see "Port 3000 is already in use".

By Dilli Babu Kadati — https://github.com/dillibabukadati/port-sweeper

Screenshots

Port Sweeper GUI — Active Ports table and Kill a Port:

![Port Sweeper GUI](https://github.com/dillibabukadati/port-sweeper/raw/main/assets/screenshot-gui.png)

Success feedback after killing a port:

![Port Sweeper success message](https://github.com/dillibabukadati/port-sweeper/raw/main/assets/screenshot-gui-success.png)

Install

  From source (requires Rust):
    cargo install --path .
    # Puts psweep and port-sweeper on your PATH

  From releases (one install gives you both GUI and CLI on all platforms):
    macOS: Download the .dmg for your chip (Intel or Apple Silicon). Open it and double‑click "Install Port Sweeper" — installs the app and configures psweep/port-sweeper for the terminal.
    Windows: Download the .exe installer. Run it and keep "Add to PATH" checked — installs the app and configures psweep/port-sweeper for the terminal.
    Linux: Download the .deb, then: sudo dpkg -i psweep_*_amd64.deb — installs both and adds them to your PATH.

Usage

  List active ports:
    psweep list

  Kill process on a port:
    psweep kill 3000

  Kill multiple ports (comma-separated or range):
    psweep kill 3000,8000,9000-9010

  Open the GUI:
    psweep gui
    # Or run the standalone GUI binary:
    port-sweeper

Building

  Prerequisites: Rust (rustup recommended).

  cargo build --release
  # Binaries: target/release/psweep, target/release/port-sweeper (or .exe on Windows)

  Cross-compile for another target:
    rustup target add <target>
    cargo build --release --target <target>

  Example targets:
    x86_64-pc-windows-msvc
    x86_64-unknown-linux-gnu
    x86_64-apple-darwin
    aarch64-apple-darwin

Platforms

  Windows (x86_64), Linux (x86_64), macOS (Intel and Apple Silicon).

  Release builds for each platform are produced by CI on every version tag (e.g. v0.1.0).

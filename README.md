# Port Sweeper (psweep)

Find and kill processes by port. A lightning-fast GUI for when you see *"Port 3000 is already in use"*.

*This release focuses on the GUI; CLI docs are commented below for a future release.*

**By [Dilli Babu Kadati](https://github.com/dillibabukadati/port-sweeper)**

---

## Screenshots

**Port Sweeper GUI — Active Ports table and Kill a Port:**

![Port Sweeper GUI](https://github.com/dillibabukadati/port-sweeper/raw/main/assets/screenshot-gui.png)

**Success feedback after killing a port:**

![Port Sweeper success message](https://github.com/dillibabukadati/port-sweeper/raw/main/assets/screenshot-gui-success.png)

---

## Install

### From releases (this release: GUI app only)

- **macOS:** Download the `.dmg` for your chip (Intel or Apple Silicon). Open it and double‑click **Port Sweeper** — installs the app to Applications (you’ll be asked for your password once).
- **Windows:** Download the `.exe` installer and run it.
- **Linux:** Download the `.deb`, then: `sudo dpkg -i psweep_*_amd64.deb`.

<!-- CLI install (uncomment for a future release):
### From source (requires Rust)
```bash
cargo install --path .
# Puts psweep and port-sweeper on your PATH
```
macOS: "configures psweep/port-sweeper for the terminal" | Windows: keep "Add to PATH" checked | Linux: adds to PATH
-->

---

## Usage

**Open Port Sweeper** from Applications (macOS/Windows) or your app menu — use the GUI to list ports and kill processes on a port.

<!-- CLI usage (uncomment for a future release):
**List active ports:** `psweep list`
**Kill process on a port:** `psweep kill 3000`
**Kill multiple (comma or range):** `psweep kill 3000,8000,9000-9010`
**Open the GUI:** `psweep gui` or `port-sweeper`
-->

---

## Building

**Prerequisites:** Rust ([rustup](https://rustup.rs) recommended).

```bash
cargo build --release
# Binaries: target/release/psweep, target/release/port-sweeper (or .exe on Windows)
```

**Cross-compile for another target:**

```bash
rustup target add <target>
cargo build --release --target <target>
```

**Example targets:**

- `x86_64-pc-windows-msvc`
- `x86_64-unknown-linux-gnu`
- `x86_64-apple-darwin`
- `aarch64-apple-darwin`

---

## Platforms

Windows (x86_64), Linux (x86_64), macOS (Intel and Apple Silicon).

Release builds for each platform are produced by CI on every version tag (e.g. `v0.1.0`).

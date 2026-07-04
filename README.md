# wl-gammactl-rust

A simple tool to adjust contrast, brightness, gamma, and saturation on Wayland outputs.

Inspired by [wl-gammactl](https://github.com/mischw/wl-gammactl/).

> **Note:** This only supports Hyprland (and other wlroots-based compositors via `wlr-gamma-control-unstable-v1`). KDE and GNOME are not supported, as they don't implement this protocol.

## Requirements

- [Cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html) (Rust's build tool/package manager)

## Installation

### From AUR

```bash
yay -S wl-gammactl-rust
```

### From source

```bash
cd /tmp
git clone https://github.com/Lecer69/wl-gammactl-rust
cd wl-gammactl-rust
cargo build --release
sudo cp target/release/wl-gammactl-rust /usr/local/bin/wl-gammactl-rust
```

## Usage

### GUI

```bash
wl-gammactl-rust
```

### CLI

```bash
wl-gammactl-rust -c 1.5 -b 1.2 -g 1.0 -s 1.0
```

Options:
- `-c, --contrast <FLOAT>` — Contrast (default: 1.0)
- `-b, --brightness <FLOAT>` — Brightness (default: 1.0)
- `-g, --gamma <FLOAT>` — Gamma (default: 1.0)
- `-s, --saturation <FLOAT>` — Saturation (default: 1.0)

## License

Unlicense (public domain)

---

This is my first Public Rust project! Thank you for using it with ❤️
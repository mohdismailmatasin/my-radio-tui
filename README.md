# My Radio TUI

A beautiful terminal-based radio player for Malaysian radio stations. Built with Rust.

![My Radio TUI](screenshot.png)

---

## Features

- **One-click playback** - Select and play radio stations instantly
- **Keyboard navigation** - Full control without a mouse
- **Beautiful interface** - Clean, modern TUI design
- **Lightweight** - Minimal resource usage

---

## Quick Start

```bash
# Build
cargo build --release

# Run
./target/release/my-radio-tui

# Install system-wide
sudo make install

# Run after install
my-radio-tui
```

---

## Controls

| Key | Action |
|-----|--------|
| `↑` / `↓` | Navigate stations |
| `PgUp` / `PgDn` | Page up / down |
| `Home` / `End` | First / last station |
| `Enter` | Play selected station |
| `Space` | Pause / Resume |
| `q` / `Esc` | Quit |

---

## Installation Options

### Option 1: Build from source
```bash
cargo build --release
sudo make install
```

### Option 2: Install via Cargo
```bash
cargo install --path .
```

---

## Uninstallation

### Option 1: Make uninstall
```bash
sudo make uninstall
```

### Option 2: Uninstall script
```bash
./uninstall.sh
```

### Option 3: Cargo uninstall
```bash
cargo uninstall my-radio-tui
```

---

## Radio Stations

| Station | Genre |
|---------|-------|
| 8 FM | Pop |
| Asyik FM | Malay |
| Best FM | Pop |
| Era FM | Contemporary |
| Fly FM | Pop |
| Hitz FM | Pop |
| Hot FM | Talk |
| Johor FM | Regional |
| Kool FM | Retro |
| Nasional FM | National |
| Radio Klasik | Classic |
| Ria FM | Malay |
| Sinar FM | Malay |
| Suria FM | Malay |

---

## Requirements

- Rust (latest stable)
- mpv player (for audio playback)

```bash
# Install mpv on Ubuntu/Debian
sudo apt install mpv

# Install mpv on Arch Linux
sudo pacman -S mpv

# Install mpv on macOS
brew install mpv
```

---

## License

MIT License - feel free to use and modify.
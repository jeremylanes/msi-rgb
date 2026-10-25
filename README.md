<p align="center">
  <img src="assets/logo.svg" alt="msi-rgb logo" width="130" height="130" />
</p>

# msi-rgb

> Native Linux RGB keyboard controller for MSI Katana 15 HX laptops.

[![Crates.io](https://img.shields.io/crates/v/msi-rgb?style=flat-square&color=ff1744)](https://crates.io/crates/msi-rgb)
[![CI Status](https://img.shields.io/github/actions/workflow/status/jeremylanes/msi-rgb/ci.yml?branch=main&style=flat-square)](https://github.com/jeremylanes/msi-rgb/actions)
[![License](https://img.shields.io/badge/License-MIT%20OR%20Apache--2.0-blue?style=flat-square)](LICENSE)
[![Docs & Live Visualizer](https://img.shields.io/badge/Docs-Live%20Visualizer-blueviolet?style=flat-square)](https://jeremylanes.github.io/msi-rgb/)

---

## Overview

When switching to Linux (Fedora) on a new MSI Katana 15 HX (B14WGK / MS-1565 / `1462:1601`), there was no native Linux utility to control the 4-zone RGB keyboard backlighting. Proprietary vendor software is Windows-only, and generic third-party tools lack direct hardware HID support for this microcontroller.

`msi-rgb` restores complete hardware control on Linux with zero bloat:

- **Direct Linux HID Driver**: Interacts directly with `/dev/hidraw*` via reverse-engineered Feature Reports (`0x01`, `0x02`, `0xA0`). Instant connection (&lt;1ms) with zero background daemon required.
- **Hardware Flash Persistence**: Emits packet `0xA0` to save colors into keyboard internal EEPROM/Flash. Lighting persists across reboots, shutdowns, and powers on at the BIOS/UEFI level without any software running.
- **Whole-Keyboard Synchronization**: Eliminates internal MCU phase-delay drift by driving Mode 1 Steady synchronously across all 4 zones (`0x0F`) at 33 FPS for butter-smooth color cycles.
- **Dual Interface**: Includes a modern GTK4 GUI with interactive chiclet key visualizer, alongside a robust CLI designed for terminal users and window manager keybinds.
- **Vibecoded**: Entirely designed and engineered with autonomous agent pair-programming using **Google Antigravity (AGY)** and the **BMad Method**.

**Interactive Simulator:** Try out colors and profiles on the live [Documentation & Hardware Simulator](https://jeremylanes.github.io/msi-rgb/).

---

## Installation

### Via Cargo (crates.io)

Ensure you have GTK4 development headers installed on your system:

- **Fedora**: `sudo dnf install gtk4-devel cairo-gobject-devel glib2-devel pkgconf-pkg-config`
- **Ubuntu / Debian**: `sudo apt install libgtk-4-dev libcairo2-dev libglib2.0-dev pkg-config`
- **Arch Linux**: `sudo pacman -S gtk4 cairo glib2 pkgconf`

Then install with Cargo:

```bash
cargo install msi-rgb
```

### 1-Line Installer via cURL (Alternative)

Installs system dependencies, compiles release binary, and configures desktop launcher and icons:

```bash
# User session installation (recommended)
curl -fsSL https://raw.githubusercontent.com/jeremylanes/msi-rgb/main/bin/install.sh | bash -s -- --user

# System-wide installation
curl -fsSL https://raw.githubusercontent.com/jeremylanes/msi-rgb/main/bin/install.sh | bash -s -- --system
```

---

## Quickstart

```bash
# 1. Apply smooth whole-keyboard spectrum wave (runs detached in background)
msi-rgb --profile "Spectrum Harmony"

# 2. Set MSI signature dragon red solid backlight (saved to hardware flash)
msi-rgb --profile "Gaming Red"

# 3. List all built-in and user profiles
msi-rgb --list-profiles

# 4. Launch the graphical studio interface
msi-rgb
```

---

## Command Reference

| Command / Flag | Description | Example |
|---|---|---|
| `msi-rgb` | Launch the GTK4 graphical studio interface | `msi-rgb` |
| `--profile <NAME>` | Apply a profile instantly (detaches background daemon for dynamic cycles) | `msi-rgb --profile "Cyberpunk Neon"` |
| `--list-profiles` | Print all configured factory and user profiles to stdout | `msi-rgb --list-profiles` |
| `--apply-default` | Apply the startup default profile configured in settings | `msi-rgb --apply-default` |
| `--daemon` | Run headless in background for software animation loops | `msi-rgb --daemon` |
| `--mock` | Run in offline simulation mode without physical hardware | `msi-rgb --mock` |

---

## Window Manager Keybindings

Add fast lighting profile switches to your `hyprland.conf`, `sway/config`, or `i3/config`:

```ini
# Hyprland example
bind = $mainMod, F5, exec, msi-rgb --profile "Spectrum Harmony"
bind = $mainMod, F6, exec, msi-rgb --profile "Gaming Red"
bind = $mainMod, F7, exec, msi-rgb --profile "Cyberpunk Neon"
bind = $mainMod, F8, exec, msi-rgb --profile "Stealth Off"
```

---

## Udev Rules (Non-Root Access)

To interact with `/dev/hidraw*` without `sudo`, install the udev rule:

```bash
sudo cp packaging/99-msi-rgb.rules /etc/udev/rules.d/
sudo udevadm control --reload-rules && sudo udevadm trigger
```

---

## License

Licensed under either of [MIT](LICENSE) or [Apache-2.0](LICENSE) at your option.

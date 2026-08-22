# MSI Mystic Light Linux RGB Controller

A modern, Apple & DaVinci-grade Linux RGB keyboard controller in **Rust** and **GTK4**, designed to replace MSI Center / Mystic Light for laptops such as the **MSI Katana 15 HX B14WGK** (`MS-1565` / USB VID:PID `1462:1601`).

---

## 🌟 Key Features

* **Apple & DaVinci-Grade UI Design**: Ultra-refined glassmorphism, frosted dark card surfaces (`backdrop-filter`), luminescent accents, responsive layout, and high-DPI typography.
* **Interactive Cairo HSV Color Wheel**:
  * Real-time continuous 360° Hue and radial Saturation disc with smooth reticle tracking.
  * Dedicated Lightness / Value slider with live color preview.
  * Direct `#RRGGBB` Hex entry with instant bidirectional sync.
  * Precise RGB channel SpinButtons (0..255).
  * Curated Pro Studio & Gaming preset palette (MSI Dragon Red, Cyber Cyan, Neon Magenta, Matrix Emerald, Vaporwave Pink, Glacier Blue, Toxic Lime, Tokyo Indigo, Warm 2700K, Daylight 6500K).
  * Customizable **Favorites Swatches Bar** to bookmark and recall custom user colors.
* **4-Zone Independent & Synchronized Control**:
  * Zone 1 (WASD & Gaming Left cluster)
  * Zone 2 (Center Alpha)
  * Zone 3 (Navigation & Return)
  * Zone 4 (Numpad)
* **Real-time Dynamic Lighting Visualizer**: Interactive visual preview with authentic chiclet key matrix and multi-layered underglow that reacts dynamically to colors, brightness, and animations.
* **Reboot & Startup Persistence**:
  * Auto-restores saved lighting profile upon Linux session login or boot via both XDG autostart (`~/.config/autostart/msi-rgb.desktop`) and systemd user unit (`~/.config/systemd/user/msi-rgb.service`).
  * Automatic resilient connection retry loop to wait for OpenRGB service startup.
  * Atomic temporary-file JSON persistence (`~/.config/msi-rgb/profiles.json`).
* **Factory & Custom Profiles Studio**:
  * Built-in presets: *Gaming Red*, *Cyberpunk Neon*, *Matrix Code*, *Arctic Ice*, *Night Shift*, *Rainbow Wave*, *Stealth Off*.
  * Create, duplicate, delete, and designate startup default profiles with 4-zone preview dots.
* **Decoupled Architecture**: Fully isolated Domain and Backend layers with `RgbBackend` trait abstraction (supporting live `OpenRgbBackend` and `MockBackend`).
* **Zero Root / Sudo Required**: Communicates safely with the local OpenRGB SDK daemon over TCP socket.

---

## 🚀 Quick Start

### 1. Launching GUI
```bash
./bin/msi-rgb
```

### 2. CLI Usage Examples
```bash
# List all configured profiles
./bin/msi-rgb --list-profiles

# Apply a profile instantly and exit
./bin/msi-rgb --profile "Cyberpunk Neon"

# Apply startup default profile (auto-retries until OpenRGB daemon is ready)
./bin/msi-rgb --apply-default

# Run in offline simulation mock mode
./bin/msi-rgb --mock

# Run in headless background daemon mode
./bin/msi-rgb --daemon
```

### 3. Hyprland / Sway Keybindings Integration
Add the following to your `hyprland.conf`:
```ini
# Fast RGB Profile Switching
bind = $mainMod, F5, exec, /path/to/msi-rgb/bin/msi-rgb --profile "Gaming Red"
bind = $mainMod, F6, exec, /path/to/msi-rgb/bin/msi-rgb --profile "Cyberpunk Neon"
bind = $mainMod, F7, exec, /path/to/msi-rgb/bin/msi-rgb --profile "Night Shift"
bind = $mainMod, F8, exec, /path/to/msi-rgb/bin/msi-rgb --profile "Stealth Off"
```

---

## 🛠 Project Structure

```text
msi_rgb/
├── bin/
│   ├── msi-rgb          # Application launcher
│   ├── run-tests        # Full verification suite (fmt, clippy, unit, doctest)
│   ├── package-rpm      # Fedora package staging helper
│   └── backup-config    # Configuration backup archiver
├── packaging/
│   ├── msi-rgb.spec     # RPM package specification
│   ├── com.msi.mysticlight.rgb.desktop
│   └── com.msi.mysticlight.rgb.svg
├── src/
│   ├── main.rs          # CLI argument parser and runtime entry point
│   ├── lib.rs           # Core library exports
│   ├── domain/          # Pure business logic (Color with HSV, Effect, Zone, Device, Profile)
│   ├── backend/         # Hardware abstraction (RgbBackend trait, OpenRGB SDK, Mock)
│   ├── effects/         # Effect models and color palettes
│   ├── profiles/        # Preset management and factory profiles
│   ├── persistence/     # Atomic XDG storage with schema migration
│   ├── settings/        # App configuration
│   ├── infrastructure/  # Logging, autostart (XDG & systemd)
│   ├── app/             # Background async worker service and state handle
│   └── ui/              # GTK4 graphical interface (Cairo HSV wheel, visualizer, studio)
└── tests/               # Comprehensive unit and integration test suites
```

---

## 🧪 Testing & Verification

Run the test suite with one command:
```bash
./bin/run-tests
```

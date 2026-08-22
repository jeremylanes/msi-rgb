# MSI Mystic Light Linux RGB Controller

A modern, native, and robust Linux RGB keyboard controller in **Rust** and **GTK4**, designed to replace MSI Center / Mystic Light for laptops such as the **MSI Katana 15 HX B14WGK** (`MS-1565` / USB VID:PID `1462:1601`).

---

## 🌟 Key Features

* **Native Wayland & Hyprland Integration**: Seamless dark-mode GTK4 interface styled in MSI gaming aesthetic.
* **4-Zone Independent & Synchronized Control**:
  * Zone 1 (WASD & Left cluster)
  * Zone 2 (Center Left)
  * Zone 3 (Center Right)
  * Zone 4 (Numpad)
* **Real-time Dynamic Lighting Preview**: Interactive visual representation with live glow and keycap previews.
* **Decoupled Architecture**: Fully isolated Domain and Backend layers with `RgbBackend` trait abstraction (supporting both live `OpenRgbBackend` and `MockBackend`).
* **Factory & Custom Profiles**:
  * Built-in presets: *Gaming Red*, *Cyberpunk Neon*, *Matrix Code*, *Arctic Ice*, *Night Shift*, *Rainbow Wave*, *Stealth Off*.
  * Create, duplicate, delete, and apply custom user profiles saved atomically to `~/.config/msi-rgb/profiles.json`.
* **Zero Root / Sudo Required**: Communicates safely with the local OpenRGB SDK daemon over TCP socket.
* **CLI & Daemon Modes**: Fast terminal commands for scripts, keybindings, and autostart on session login.

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

# Apply startup default profile
./bin/msi-rgb --apply-default

# Run in offline simulation mock mode
./bin/msi-rgb --mock

# Run in headless background daemon mode
./bin/msi-rgb --daemon
```

### 3. Hyprland Keybindings Integration
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
msi_free_center/
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
│   ├── domain/          # Pure business logic (Color, Effect, Zone, Device, Profile)
│   ├── backend/         # Hardware abstraction (RgbBackend trait, OpenRGB SDK, Mock)
│   ├── effects/         # Effect models and color palettes
│   ├── profiles/        # Preset management and factory profiles
│   ├── persistence/     # Atomic XDG storage with schema migration
│   ├── settings/        # App configuration
│   ├── infrastructure/  # Logging, autostart, systemd
│   ├── app/             # Background async worker service and state handle
│   └── ui/              # GTK4 graphical interface and visualizer
└── tests/               # Comprehensive unit and integration test suites
```

---

## 🧪 Testing & Verification

Run the test suite with one command:
```bash
./bin/run-tests
```

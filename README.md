# Cruce

Cruce is a lightweight Linux launcher for Windows games built on **UMU** (`umu-launcher`), **Proton**, and **Wine**, implemented in **Rust** and **Tauri v2**.

It provides a standalone, storefront-independent launcher for managing and launching Windows executables with isolated Wine prefixes, Gamescope integration, automated save backups, and controller navigation.

---

## Motivation & Prior Art

Cruce is a ground-up native rewrite of [Crucible](https://github.com/Adarsh219/cruce), inspired by its clean workflow and transparent filesystem architecture. 

Crucible proved the viability of a dedicated UMU-native launcher without storefront dependencies, but relied on Python 3 and `PyQt6-WebEngine`, which bundled Chromium and carried high memory overhead. Cruce preserves Crucible's exact configuration schema and prefix structure while replacing the Python/Qt runtime with a native Rust core:

* **100% Drop-In Compatibility**: Reads and writes to `~/.local/share/crucible-launcher/` with zero migration needed. Existing game configurations, save backups, and prefixes are recognized automatically.
* **Minimal Resource Footprint**: Idles at ~25 MB – 40 MB Resident Set Size (RSS), keeping host memory available for game processes.
* **Low Startup Latency**: Instant cold launch (<70ms), avoiding Python module import and Qt runtime initialization.
* **Native Process Supervision**: Replaces periodic `/proc` environ polling loops with kernel `pidfd` descriptors and POSIX double-fork detachment.

---

## Performance Comparison

Measurements taken on Arch Linux (Kernel 6.12, AMD Ryzen 7 7840U, 32 GB RAM):

| Metric | Crucible (Python 3 + PyQt6) | Cruce (Rust + Tauri v2) | Measurement Method |
| :--- | :--- | :--- | :--- |
| **Cold Startup** | 2,140 ms ± 85 ms | **58 ms ± 4 ms** | `hyperfine --warmup 3 'cruce'` |
| **Idle Memory (RSS)** | 384 MB | **32 MB** | `ps -o rss= -p <pid>` after window draw |
| **Installed Binary Size** | ~260 MB (Python + Qt6 wheels) | **~19 MB** (Stripped ELF + assets) | `du -sh` |
| **Process Tracking Overhead** | Periodic `/proc` polling spikes | **$O(1)$ Event-driven (`pidfd`)** | Kernel `perf stat` |

---

## Core Capabilities

* **Execution Pipeline**: Wraps games in an optimized hierarchy:
  `[gamescope] ➔ [systemd-run] ➔ [mangohud] ➔ [gamemoderun] ➔ [wrapper] ➔ [bwrap] ➔ [umu-run] ➔ [game.exe] ➔ [args]`
* **POSIX Double-Fork Detachment**: Reparents child game processes to PID 1, ensuring proper window grouping under KDE Plasma, GNOME, and Wayland compositors.
* **In-Memory PE Icon Extraction**: Extracts embedded high-resolution `.ico` and `.png` icons directly from Windows PE32/PE32+ binaries via `pelite` without external tools.
* **Denuvo Anti-Tamper Freeze**: Uses Bubblewrap (`bwrap`) to snapshot and bind-mount `/proc/cpuinfo` and `/proc/version`, preventing hardware fingerprint invalidation across host kernel updates.
* **Wine Power Tools**: One-click access to `winecfg`, `regedit`, `taskmgr`, `cmd`, and `winetricks` scoped directly to the selected prefix.
* **Save Backups**: Automated `.zip` compression and restoration of Wine prefix save directories.
* **Safe Runner Extraction**: Multi-threaded downloader for GE-Proton, UMU-Proton, and CachyOS tarballs with strict Zip-Slip and symlink path-traversal protection.
* **Controller Navigation**: Complete 60 FPS Gamepad API navigation loop with configurable stick deadzones and Steam Deck button mappings.

---

## Project Structure

```
cruce/
├── crates/
│   ├── cruce-core/        # Pure Rust library: process supervisor, config store, launch pipeline
│   └── cruce-app/         # Tauri v2 desktop shell and typed IPC command surface
├── ui/                    # React 19 + TypeScript + Tailwind CSS v4 frontend
├── packaging/             # Desktop integration (.desktop, icon assets, install scripts)
├── scripts/               # Packaging and distribution build helpers
└── ARCHITECTURE.md        # Technical specification and architecture notes
```

---

## Building from Source

### Prerequisites

* Rust toolchain (1.80+)
* Node.js (20+) & npm
* `libwebkit2gtk-4.1` (or `4.0`)
* `pkg-config`

### Build Instructions

```bash
# Clone the repository
git clone https://github.com/Adarsh219/cruce.git
cd cruce

# Run test suites (42 Rust unit/integration tests)
cargo test --workspace --all-targets

# Build frontend bundle
npm --prefix ui run build

# Run in development mode
cargo tauri dev
```

### Packaging

Release packages can be generated via the helper scripts in `scripts/`:

```bash
# Build Debian package (.deb) in dist/
./scripts/build-deb.sh

# Build standalone distribution tarball (.tar.gz) in dist/
./scripts/build-tarball.sh
```

---

## Controller Navigation (Gamepad / Steam Deck)

| Control | Action |
| :--- | :--- |
| **D-Pad / Left Stick** | Navigate game cards and modal options |
| **(A)** | Launch Game / Confirm Selection |
| **(B)** | Close Modal / Go Back |
| **(X)** | Toggle Play / Stop Active Game |
| **(Y)** | Open Game Settings Modal |
| **LB / RB** | Cycle Views (Library, Wine Cabinet, Settings) |
| **Start / Select** | Open Settings View / Add Game Dialog |

---

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT License](LICENSE-MIT) at your option.

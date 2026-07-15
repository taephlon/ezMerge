# ezMerge: The Modern Companion for Gentoo and Portage

`ezMerge` is an ecosystem designed to make Gentoo overlays, dependency resolution, and Portage package installation effortless for beginners and power users alike, without hiding Portage's underlying power.

Rather than hiding the compilation and overlay mechanisms, ezMerge explains what it is doing, asks for permission before making configuration changes, and leaves the system in a standard Gentoo-compatible state.

---

## 🏗️ Repository Layout

The workspace is organized into a monorepo containing several interconnected components:

- **`ezmerge-cli/`**: The command-line assistant written in Rust. It interacts with `/etc/portage` to manage configurations, unmask keywords, customize USE flags, and run Portage.
- **`ezmerge-overlay/`**: A curated Gentoo overlay repository skeleton showing how overlay categories, repository names, layout configurations, and Rust ebuilds are defined.
- **`ezmerge-web/`**: A rich, dark-themed, glassmorphic search portal that queries the package list and demonstrates dependency graphs, licenses, and interactive USE flag selection.
- **`ezmerge-api/`**: Contains the central package metadata database (`db.json`) that powers both the web portal and the CLI.
- **`ezmerge-docs/`**: Markdown guides explaining installation, Portage mechanics, and packaging rules.

---

## ⚙️ Dependencies & System Requirements

Before installing, please ensure your system meets the requirements:

### Required Dependencies
* **Rust & Cargo** (v1.80+): Required for compiling the `ezmerge-cli` binary from source.
* **Unix-like Environment**: Standard `libc` interface (tested on Linux/Gentoo).

### Optional / Integration Dependencies
* **Gentoo Portage (`emerge` & `/etc/portage`)**: Required for live Gentoo overlay configuration, package unmasking, and package installation.
  > [!NOTE]
  > If `emerge` is not found, `ezMerge` automatically executes in a **mock/simulation mode**. This allows testing, development, and sandbox usage on non-Gentoo systems (e.g. Debian, Fedora, Arch).
* **Python 3**: Required to run the local API & package search portal (`server.py`).
* **`eselect-repository`**: Used to search and dynamically enable Gentoo overlays.

### ⚠️ Conflict Warnings & Package Precautions
* **Legacy `layman`**: `ezMerge` manages overlays via `eselect-repository` (`/etc/portage/repos.conf/`). If you use layman, we recommend migrating your overlays to eselect-repository to prevent profile/configuration conflicts.
* **Root Permissions**: Running actual Portage merges and modifying configuration files under `/etc/portage` require root/sudo access. If run as a standard user, configuration changes are saved under the user directory (`~/.config/ezmerge/portage`) and a warning is displayed.

---

## 📥 Installation

You can install `ezMerge` using the automated script or the `Makefile`.

### Method 1: Automated Script (Recommended)
Run the installer to check system dependencies, verify the Portage environment, check for conflicts, compile the binary, and choose global or local installation:
```bash
chmod +x install.sh
./install.sh
```

### Method 2: Makefile
Alternatively, you can compile and install using standard `make` targets:
```bash
# Compile and build the release binary
make

# Install globally to /usr/local/bin (requires sudo)
sudo make install

# Uninstall from system
sudo make uninstall

# Run diagnostics
make doctor

# Clean build artifacts
make clean
```

---

## 🚀 Getting Started

### 1. Build and Run the CLI (`ezmerge-cli`)

If you installed via `install.sh` or `make install`, you can run the system-wide command:
```bash
# Run diagnostics on your Portage environment
ezmerge doctor

# Search for a package across standard overlays
ezmerge search obs

# Show details, homepage, license, and USE flags for a package
ezmerge info hyprland

# Interactively install a package
ezmerge install wezterm

# Rollback any ezMerge-applied Portage config overrides
ezmerge undo
```

Or you can run the locally built binary directly from the repo directory:
```bash
# Compile the workspace manually
cargo build --release

# Run locally
./target/release/ezmerge-cli doctor
```

### 2. Launch the Package Search Portal (`ezmerge-web`)

To start the web server hosting the package lookup directory and API:

```bash
# Run the local python web server or use 'make web'
make web
```


Now open **`http://localhost:8080`** in your browser. You can search for packages, toggle USE flags interactively, view active overlay ratings, and copy installation commands.

---

## 🎨 Design Philosophy

- **Aesthetics**: The web portal uses a terminal-inspired dark slate theme (`#0b0f19`) with glowing Gentoo Purple (`#7e5bef`) and Neon Teal (`#00f2fe`) highlights.
- **Transparency**: Every command shows the exact underlying Portage command (e.g. `eselect repository enable guru` or `emerge -av app-editors/neovim`) before proceeding.
- **Clean State**: Configuration files are written to isolated locations (e.g. `/etc/portage/package.use/ezmerge` and `/etc/portage/package.accept_keywords/ezmerge`).

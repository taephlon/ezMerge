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

## 🚀 Getting Started

### 1. Build and Run the CLI (`ezmerge-cli`)

Make sure you have Rust/Cargo installed, then compile and run the binary:

```bash
# Compile the workspace
cargo build --release

# Run diagnostics on your Portage environment
./target/release/ezmerge doctor

# Search for a package across standard overlays
./target/release/ezmerge search obs

# Show details, homepage, license, and USE flags for a package
./target/release/ezmerge info hyprland

# Interactively install a package (guides overlay add, keyword accept, and USE flag selection)
./target/release/ezmerge install wezterm

# Rollback any ezMerge-applied Portage config overrides
./target/release/ezmerge undo
```

### 2. Launch the Package Search Portal (`ezmerge-web`)

To start the web server hosting the package lookup directory and API:

```bash
# Run the local python web server
python3 server.py
```

Now open **`http://localhost:8080`** in your browser. You can search for packages, toggle USE flags interactively, view active overlay ratings, and copy installation commands.

---

## 🎨 Design Philosophy

- **Aesthetics**: The web portal uses a terminal-inspired dark slate theme (`#0b0f19`) with glowing Gentoo Purple (`#7e5bef`) and Neon Teal (`#00f2fe`) highlights.
- **Transparency**: Every command shows the exact underlying Portage command (e.g. `eselect repository enable guru` or `emerge -av app-editors/neovim`) before proceeding.
- **Clean State**: Configuration files are written to isolated locations (e.g. `/etc/portage/package.use/ezmerge` and `/etc/portage/package.accept_keywords/ezmerge`).

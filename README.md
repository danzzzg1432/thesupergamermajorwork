# Pythoneer Game

This repository contains **Pythoneer**, a small experimental game written in Rust using the [Bevy](https://bevyengine.org) engine and Python bindings provided by [pyo3](https://pyo3.rs).

## Prerequisites

- **Rust nightly** – install via [rustup](https://rustup.rs) and set the toolchain with:
  ```bash
  rustup toolchain install nightly
  rustup override set nightly
  ```
- **Python 3.13** – required by `pyo3` for the embedded Python runtime.
- On Linux, you will also need development packages for audio and graphics such as `vulkan-loader`, `alsa-lib`, `udev` and `wayland`.
  These are provided automatically when using the provided `shell.nix` with Nix.

## Building and Running

Clone the repository and run the game inside the `game` directory:

```bash
git clone <repository_url>
cd thesupergamermajorwork/game
cargo run
```

This builds and launches the game in debug mode. Use `cargo run --release` to build an optimized release version.

## Using Nix (optional)

If you have [Nix](https://nixos.org) installed you can enter a development shell that provides all dependencies:

```bash
cd game
nix develop   # or `nix-shell` on older versions
cargo run
```

The shell sets up the required Rust toolchain, system libraries and Python version automatically.


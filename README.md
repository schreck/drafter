# Drafter

A STEP file viewer written in Rust. Parses AP214 `.stp` files, tessellates B-rep edge geometry, and displays a wireframe front elevation in an SDL2/OpenGL 3.3 window.

## Dependencies

### Rust

Install Rust via [rustup](https://rustup.rs/):

```sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Requires Rust 1.93 or later.

### SDL2

The window is powered by SDL2 via the [`beryllium`](https://crates.io/crates/beryllium) crate, which requires the SDL2 native library to be installed on your system.

**Ubuntu/Debian:**
```sh
sudo apt install libsdl2-dev
```

**macOS (Homebrew):**
```sh
brew install sdl2
```

**Windows:**
Download the SDL2 development libraries from https://github.com/libsdl-org/SDL/releases and follow the setup instructions for your toolchain.

## Building & Running

```sh
cargo run
```

Close the window or press the OS close button to exit.

You are helping the user work with beryllium windows in this Rust project.

## Project context

- Crate: `beryllium = "0.2.1"` (SDL2 wrapper)
- SDL2 system lib required: `libsdl2-dev` (Linux), `brew install sdl2` (macOS)
- Entry point: `src/main.rs`

## Key API facts (beryllium 0.2.1)

**Initialization**
```rust
use beryllium::{SDL, InitFlags};
let sdl = SDL::init(InitFlags::Everything).expect("SDL init failed");
```

**GL attributes** — must be set before creating the window
```rust
use beryllium::SdlGlAttr;
sdl.gl_set_attribute(SdlGlAttr::MajorVersion, 3).unwrap();
sdl.gl_set_attribute(SdlGlAttr::MinorVersion, 3).unwrap();
sdl.gl_set_attribute(SdlGlAttr::Profile, 1).unwrap(); // 1 = Core, 2 = Compatibility
```

**Window creation**
```rust
use beryllium::{WindowFlags, WindowPosition};
let win = sdl
    .create_gl_window("Title", WindowPosition::Centered, 800, 600, WindowFlags::Shown)
    .expect("window creation failed");
```
- `WindowPosition` variants: `Centered`, `Undefined`, `XY(x, y)`
- `WindowFlags` constants: `Shown`, `OpenGL`, `Vulkan` (combinable with `|`)
- Only one window is allowed at a time

**Event loop**
```rust
use beryllium::Event;
'main_loop: loop {
    while let Some(event) = sdl.poll_events().and_then(Result::ok) {
        match event {
            Event::Quit(_) => break 'main_loop,
            Event::Keyboard(key) => { /* handle key */ }
            Event::MouseButton(btn) => { /* handle mouse */ }
            _ => {}
        }
    }
    // render here
}
```

**Event variants:** `Quit`, `Window`, `Keyboard`, `MouseMotion`, `MouseButton`, `MouseWheel`, `ControllerDevice`, `ControllerButton`, `ControllerAxis`

**GL swap** (after rendering)
```rust
win.swap_window();
```

## Common tasks

When the user asks to:
- **Add keyboard input** — match on `Event::Keyboard(key)` and use `key.keycode`
- **Add a render loop** — call OpenGL draw calls inside the loop, then `win.swap_window()`
- **Change window size or title** — update args to `create_gl_window` or call `win.win.set_title("new title")`
- **Load OpenGL functions** — recommend adding the `gl` or `glow` crate and loading with `gl::load_with(|s| win.get_proc_address(s))`
- **Add delta time** — use `sdl.get_ticks()` which returns milliseconds since init

## What to avoid
- Do not use the old tutorial API (`Sdl::init`, `init::InitFlags::EVERYTHING`, `video::GlProfile`, etc.) — it is outdated and does not match beryllium 0.2.1
- Do not create more than one window
- Always set GL attributes before calling `create_gl_window`

$ARGUMENTS

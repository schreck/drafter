# Drafter — Agent Guide

A Rust/OpenGL learning project following the [learn-opengl](https://rust-tutorials.github.io/learn-opengl/) tutorial series, adapted for the versions of beryllium and ogl33 actually available on crates.io.

## Stack

| Crate | Version | Purpose |
|---|---|---|
| `beryllium` | 0.2.1 | SDL2 window + GL context |
| `ogl33` | 0.2.0 | OpenGL 3.3 bindings |
| `bytemuck` | 1 | Safe byte casting for vertex data |

**System dependency:** SDL2 must be installed (`libsdl2-dev` on Linux, `brew install sdl2` on macOS).

## Project structure

```
src/
  main.rs       — entry point, SDL init, window, event loop, render
  gl_utils.rs   — thin safe wrappers over raw ogl33 calls
```

### gl_utils.rs exports

- `VertexArray` — VAO wrapper
- `Buffer` + `BufferType` — VBO/EBO wrapper (`Array`, `ElementArray`)
- `ShaderProgram::from_vert_frag(vert, frag)` — compiles and links shaders
- `PolygonMode` + `polygon_mode(mode)` — fill/line/point rendering
- `clear_color(r, g, b, a)` — sets glClearColor
- `buffer_data(ty, &[u8], usage)` — uploads data; use `bytemuck::cast_slice` to convert vertex arrays

## Critical: beryllium 0.2.1 API

The online tutorials use an **older, incompatible beryllium API**. Always use the correct 0.2.1 API:

```rust
// CORRECT
let sdl = SDL::init(InitFlags::Everything).unwrap();
sdl.gl_set_attribute(SdlGlAttr::Profile, GlProfile::Core).unwrap();
let win = sdl.create_gl_window("Title", WindowPosition::Centered, 800, 600, WindowFlags::Shown).unwrap();
while let Some(event) = sdl.poll_events().and_then(Result::ok) { ... }
Event::Quit(_) => break   // Quit wraps a QuitEvent value
Event::MouseMotion(e) => { e.x_pos; e.y_pos; }

// WRONG (old API, will not compile)
Sdl::init(InitFlags::EVERYTHING)
sdl.set_gl_profile(GlProfile::Core)
sdl.create_gl_window(CreateWinArgs { .. })
sdl.poll_events() -> Option<(Event, timestamp)>
Event::Quit  // no inner value
```

`win.set_swap_interval(SwapInterval::Vsync)` returns `i32`, **not** `Result` — do not `.unwrap()` it.

## Coordinate system

SDL and OpenGL use opposite Y directions. To convert mouse screen coords → OpenGL NDC:

```rust
let ndc_x = (mouse_x / WIN_W) * 2.0 - 1.0;
let ndc_y = 1.0 - (mouse_y / WIN_H) * 2.0;
```

## Running

```sh
cargo run
```

## Skills

Use `/beryllium` for detailed API reference and code patterns for this project.

You are helping the user work with beryllium windows and OpenGL in this Rust project.

## Project context

- Crate: `beryllium = "0.2.1"` (SDL2 wrapper)
- Crate: `ogl33 = "0.2.0"` (OpenGL 3.3 bindings)
- Crate: `bytemuck = "1"` (safe byte casting for vertex data)
- SDL2 system lib required: `libsdl2-dev` (Linux), `brew install sdl2` (macOS)
- Entry point: `src/main.rs`
- GL helper wrappers: `src/gl_utils.rs` (VertexArray, Buffer, BufferType, Shader, ShaderProgram, PolygonMode)

## Key API facts (beryllium 0.2.1)

**Initialization**
```rust
use beryllium::{SDL, InitFlags};
let sdl = SDL::init(InitFlags::Everything).expect("SDL init failed");
```

**GL attributes** — must be set before creating the window
```rust
use beryllium::{SdlGlAttr, GlProfile, ContextFlag};
sdl.gl_set_attribute(SdlGlAttr::MajorVersion, 3).unwrap();
sdl.gl_set_attribute(SdlGlAttr::MinorVersion, 3).unwrap();
sdl.gl_set_attribute(SdlGlAttr::Profile, GlProfile::Core).unwrap();
// Optional context flags (combine with |):
let mut flags = 0_i32;
if cfg!(target_os = "macos") { flags |= ContextFlag::ForwardCompatible; }
if cfg!(debug_assertions) { flags |= ContextFlag::Debug; }
sdl.gl_set_attribute(SdlGlAttr::Flags, flags).unwrap();
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

**GL swap and vsync**
```rust
use beryllium::SwapInterval;
win.set_swap_interval(SwapInterval::Vsync); // returns i32, not Result
win.swap_window();
```

**Event loop**
```rust
use beryllium::Event;
'main_loop: loop {
    while let Some(event) = sdl.poll_events().and_then(Result::ok) {
        match event {
            Event::Quit(_) => break 'main_loop,
            Event::Keyboard(key) => { /* key.keycode */ }
            Event::MouseMotion(e) => { /* e.x_pos, e.y_pos, e.x_delta, e.y_delta */ }
            Event::MouseButton(btn) => { /* btn.button, btn.x_pos, btn.y_pos */ }
            _ => {}
        }
    }
    // render here
}
```

**Event variants:** `Quit`, `Window`, `Keyboard`, `MouseMotion`, `MouseButton`, `MouseWheel`, `ControllerDevice`, `ControllerButton`, `ControllerAxis`

**MouseMotionEvent fields:** `x_pos: i32`, `y_pos: i32`, `x_delta: i32`, `y_delta: i32`, `mouse_id`, `window_id`, `timestamp`

---

## Loading OpenGL (ogl33)

```rust
use ogl33::*;
unsafe { load_gl_with(|f_name| win.get_proc_address(f_name.cast())); }
```
Call once after window creation, before any GL calls.

## gl_utils module (src/gl_utils.rs)

This project has helper wrappers — prefer these over raw ogl33 calls:

```rust
use crate::gl_utils::{
    buffer_data, clear_color, polygon_mode,
    Buffer, BufferType, PolygonMode, ShaderProgram, VertexArray,
};
```

**VertexArray** — wraps VAO
```rust
let vao = VertexArray::new().expect("Couldn't make a VAO");
vao.bind();
```

**Buffer + BufferType** — wraps VBO/EBO
```rust
let vbo = Buffer::new().expect("Couldn't make VBO");
vbo.bind(BufferType::Array);
buffer_data(BufferType::Array, bytemuck::cast_slice(&VERTICES), GL_STATIC_DRAW);

let ebo = Buffer::new().expect("Couldn't make EBO");
ebo.bind(BufferType::ElementArray);
buffer_data(BufferType::ElementArray, bytemuck::cast_slice(&INDICES), GL_STATIC_DRAW);
```

**ShaderProgram** — compiles vert+frag and links in one call
```rust
let shader_program = ShaderProgram::from_vert_frag(VERT_SHADER, FRAG_SHADER).unwrap();
shader_program.use_program();
```

**PolygonMode** — fill/wireframe/points
```rust
polygon_mode(PolygonMode::Line); // wireframe
polygon_mode(PolygonMode::Fill); // solid (default)
```

---

## Uniforms

```rust
// Get location (use null-terminated byte string)
let loc = unsafe { glGetUniformLocation(shader_program.0, b"my_uniform\0".as_ptr().cast()) };

// Set per-frame before draw
unsafe { glUniform2f(loc, x, y); }       // vec2
unsafe { glUniform1f(loc, value); }       // float
unsafe { glUniform3f(loc, r, g, b); }     // vec3
unsafe { glUniform4f(loc, r, g, b, a); }  // vec4
```

Example — mouse position as NDC uniform:
```rust
// Screen to NDC conversion (SDL y-axis is flipped vs OpenGL)
let ndc_x = (mouse_x / WIN_W) * 2.0 - 1.0;
let ndc_y = 1.0 - (mouse_y / WIN_H) * 2.0;
unsafe { glUniform2f(offset_loc, ndc_x, ndc_y); }
```

## Draw calls

```rust
unsafe {
    glClear(GL_COLOR_BUFFER_BIT);
    glDrawArrays(GL_TRIANGLES, 0, 3);                              // vertex count
    glDrawElements(GL_TRIANGLES, 6, GL_UNSIGNED_INT, 0 as *const _); // index count
}
win.swap_window();
```

---

## Common tasks

When the user asks to:
- **Add keyboard input** — match on `Event::Keyboard(key)` and use `key.keycode`
- **Track mouse position** — match on `Event::MouseMotion(e)`, use `e.x_pos` / `e.y_pos`
- **Pass data to shader** — use `glGetUniformLocation` + `glUniform*f`
- **Convert screen to GL coords** — `ndc_x = (x/W)*2-1`, `ndc_y = 1-(y/H)*2`
- **Draw indexed geometry** — use EBO + `glDrawElements`
- **Wireframe mode** — `polygon_mode(PolygonMode::Line)`
- **Add a render loop** — place GL draw calls + `win.swap_window()` at the bottom of the loop
- **Set background color** — `clear_color(r, g, b, 1.0)` before the loop (0.0–1.0)
- **Add delta time** — `sdl.get_ticks()` returns milliseconds as u32 since init

## What to avoid
- Do NOT use the old tutorial API: `Sdl::init`, `init::InitFlags::EVERYTHING`, `video::GlProfile`, `video::CreateWinArgs`, `events::Event::Quit` (no inner value) — these are from a different beryllium version and will not compile
- Do not create more than one window (beryllium 0.2.1 enforces this)
- Always set GL attributes before calling `create_gl_window`
- `win.set_swap_interval()` returns `i32`, not `Result` — do not call `.unwrap()` on it

$ARGUMENTS

You are helping the user work with beryllium windows in this Rust project.

## Project context

- Crate: `beryllium = "0.2.1"` (SDL2 wrapper)
- Crate: `ogl33 = "0.2.0"` (OpenGL 3.3 bindings)
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
use beryllium::{SdlGlAttr, GlProfile, ContextFlag};
sdl.gl_set_attribute(SdlGlAttr::MajorVersion, 3).unwrap();
sdl.gl_set_attribute(SdlGlAttr::MinorVersion, 3).unwrap();
sdl.gl_set_attribute(SdlGlAttr::Profile, GlProfile::Core).unwrap();
// Optional context flags (combine with |):
// ContextFlag::Debug, ContextFlag::ForwardCompatible
sdl.gl_set_attribute(SdlGlAttr::Flags, ContextFlag::Debug).unwrap();
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

**GL swap and vsync**
```rust
use beryllium::SwapInterval;
win.set_swap_interval(SwapInterval::Vsync); // returns i32, not Result
win.swap_window();
```

**Loading OpenGL functions with ogl33**
```rust
use ogl33::*;
unsafe {
    load_gl_with(|f_name| win.get_proc_address(f_name.cast()));
}
```
Call this once after window creation, before any GL calls.

**VAO/VBO setup (ogl33)**
```rust
unsafe {
    let mut vao = 0;
    glGenVertexArrays(1, &mut vao);
    glBindVertexArray(vao);

    let mut vbo = 0;
    glGenBuffers(1, &mut vbo);
    glBindBuffer(GL_ARRAY_BUFFER, vbo);
    glBufferData(
        GL_ARRAY_BUFFER,
        size_of_val(&VERTICES) as isize,
        VERTICES.as_ptr().cast(),
        GL_STATIC_DRAW,
    );

    glVertexAttribPointer(0, 3, GL_FLOAT, GL_FALSE,
        size_of::<Vertex>().try_into().unwrap(), 0 as *const _);
    glEnableVertexAttribArray(0);
}
```

**Shader compilation (ogl33)**
```rust
unsafe {
    let shader = glCreateShader(GL_VERTEX_SHADER); // or GL_FRAGMENT_SHADER
    glShaderSource(shader, 1,
        &(SRC.as_bytes().as_ptr().cast()),
        &(SRC.len().try_into().unwrap()));
    glCompileShader(shader);
    let mut success = 0;
    glGetShaderiv(shader, GL_COMPILE_STATUS, &mut success);
    if success == 0 {
        let mut v: Vec<u8> = Vec::with_capacity(1024);
        let mut log_len = 0_i32;
        glGetShaderInfoLog(shader, 1024, &mut log_len, v.as_mut_ptr().cast());
        v.set_len(log_len.try_into().unwrap());
        panic!("Shader error: {}", String::from_utf8_lossy(&v));
    }
}
```

**Draw call**
```rust
unsafe {
    glClear(GL_COLOR_BUFFER_BIT);
    glDrawArrays(GL_TRIANGLES, 0, 3); // last arg = vertex count
}
win.swap_window();
```

## Common tasks

When the user asks to:
- **Add keyboard input** — match on `Event::Keyboard(key)` and use `key.keycode`
- **Add a render loop** — place GL draw calls + `win.swap_window()` at the bottom of the loop
- **Change window size or title** — update args to `create_gl_window` or call `win.set_title("new title")`
- **Load OpenGL functions** — use `ogl33`: `load_gl_with(|f| win.get_proc_address(f.cast()))`
- **Add delta time** — use `sdl.get_ticks()` which returns milliseconds since init
- **Set background color** — `glClearColor(r, g, b, a)` before the loop, all values 0.0–1.0

## What to avoid
- Do not use the old tutorial API (`Sdl::init`, `init::InitFlags::EVERYTHING`, `video::GlProfile`, etc.) — it is outdated and does not match beryllium 0.2.1
- Do not create more than one window
- Always set GL attributes before calling `create_gl_window`

$ARGUMENTS

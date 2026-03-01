# Drafter — Agent Guide

A Rust/OpenGL STEP file viewer. Parses AP214 `.stp` files, tessellates
B-rep edge geometry, and displays a static wireframe front elevation in
an SDL2/OpenGL 3.3 window.

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
  main.rs          — SDL init, window, GL loop; loads STEP file and renders
  gl_utils.rs      — thin safe wrappers over raw ogl33 calls
  step_parser.rs   — AP214 STEP entity parser → HashMap<u32, Entity>
  step_geometry.rs — walks entity graph → flat Vec<[f32;3]> of GL_LINES vertices
.data/
  as1-ac-214.stp   — NIST AS1 assembly sample (AutoCAD 2000 export)
  io1-ug-214.stp   — single-part sample (Unigraphics export)
```

The active file is set by `STEP_FILE` in `main.rs`.

## STEP parser (`step_parser.rs`)

Parses a subset of AP214 entity types into `HashMap<u32, Entity>`.

**Handled entity types:** `CARTESIAN_POINT`, `DIRECTION`, `VECTOR`,
`AXIS2_PLACEMENT_3D`, `VERTEX_POINT`, `LINE`, `CIRCLE`, `EDGE_CURVE`,
`ORIENTED_EDGE`, `EDGE_LOOP`, `FACE_OUTER_BOUND`, `FACE_BOUND`,
`ADVANCED_FACE`, `CLOSED_SHELL`, `MANIFOLD_SOLID_BREP`,
`ITEM_DEFINED_TRANSFORMATION`.

**Format notes:**
- Entities can span multiple lines (Unigraphics style); the parser
  buffers lines until it sees a `;` terminator.
- Floats may have no trailing zero: `18.` is valid.
- Compound entities `#N=(TYPE1(...)TYPE2(...))` are skipped.

## Geometry extraction (`step_geometry.rs`)

`extract_segments(entities)` iterates all `EdgeCurve` entities and
returns a flat `Vec<[f32; 3]>` where every consecutive pair is one
`GL_LINES` segment. No assembly transforms are applied; geometry is
rendered in file-local coordinates.

- **LINE** edges → one segment (start vertex, end vertex)
- **CIRCLE** edges → 32 tessellated segments; arc direction is CCW;
  full circles detected when `start_id == end_id`
- Unknown curve types → chord fallback

## Rendering (`main.rs`)

`project_elevation` drops Y and maps XZ to NDC `[-0.9, 0.9]` (front
elevation view). Geometry is uploaded once at startup; the render loop
just calls `glDrawArrays(GL_LINES, ...)` with no uniforms.

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
cargo test
```

## Skills

Use `/beryllium` for detailed API reference and code patterns for this project.

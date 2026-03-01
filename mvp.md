# MVP Plan: STEP File Viewer

## Goal

Display a static wireframe **front elevation** (orthographic projection onto the XZ plane,
looking along −Y) of `as1-ac-214.stp` in the existing 800×600 SDL/OpenGL window.
No interactivity. No surface shading. Closes on Quit event only.

---

## About the File

`as1-ac-214.stp` is the classic NIST AS1 assembly sample, exported from AutoCAD 2000 in
AP214 format (millimetre units). It is a multi-part assembly containing:

| Product | Entity | Description |
|---|---|---|
| AS1-AC-214 | root assembly | top-level |
| PLATE | #70 / BREP #669 | flat plate with 6 cylindrical holes |
| L-BRACKET ASSEMBLY | #699 | sub-assembly of two L-brackets + bolts |
| (bolt, nut, washer…) | #1255, #1266, #1467, #1784, #1795 | fastener parts |

Each part's geometry is stored in local coordinates and placed into the assembly via a
rigid-body `ITEM_DEFINED_TRANSFORMATION` (a 3D axis placement).

The B-rep geometry uses two curve types on edges:
- `LINE` — straight edge between two vertices
- `CIRCLE` — circular arc (full or partial) on a cylindrical or planar face

Surface types include `CYLINDRICAL_SURFACE`, `PLANE`, and `CONICAL_SURFACE`, but for a
wireframe view we only need the **edges**, not the faces.

---

## Architecture: New Source Modules

```
src/
  main.rs          — existing window/GL loop (to be modified)
  gl_utils.rs      — existing GL wrappers (unchanged)
  step_parser.rs   — NEW: tokenise and map STEP entities
  step_geometry.rs — NEW: walk entity graph → 3D line segments
```

---

## Step 1 — STEP Parser (`src/step_parser.rs`)

### Format

Every data entity occupies one line:

```
#N=TYPE(arg, arg, ...);
```

or a compound (multiple types on one entity):

```
#N=(TYPE1(args)TYPE2(args));
```

### What to parse

We only need a small subset of the ~50 entity types present. Parse each line into an enum,
store in `HashMap<u32, Entity>`:

```rust
pub enum Entity {
    CartesianPoint([f64; 3]),
    Direction([f64; 3]),
    Vector { dir: u32, magnitude: f64 },
    Axis2Placement3D { location: u32, axis: u32, ref_dir: u32 },
    VertexPoint(u32),                         // → CartesianPoint
    Line { point: u32, dir: u32 },
    Circle { placement: u32, radius: f64 },
    EdgeCurve { start: u32, end: u32, geom: u32 },
    OrientedEdge { edge: u32, sense: bool },
    EdgeLoop(Vec<u32>),                       // → OrientedEdge list
    FaceBound { loop_: u32 },
    AdvancedFace { bounds: Vec<u32> },        // → FaceBound list
    ClosedShell(Vec<u32>),                    // → AdvancedFace list
    ManifoldSolidBrep(u32),                   // → ClosedShell
    ItemDefinedTransformation { from: u32, to: u32 }, // axis placements
    // Everything else: Unknown (skip)
}
```

### Parser approach

- Read file with `std::fs::read_to_string`
- Split on newlines; skip `HEADER`/`ENDSEC`/`END-ISO` sections
- For each `DATA` line: strip the `#N=` prefix with `split_once('=')`
- Parse `TYPE(...)` with a hand-written recursive-descent or a simple
  `split_once('(')` + careful paren-balanced substring extraction
- Parse numeric lists `(x,y,z)` with `split(',')` + `parse::<f64>()`
- Parse `#ref` with `trim_start_matches('#').parse::<u32>()`
- Ignore compound entities (`#N=(...)`) for now — they appear only in
  representation-relationship records we don't need for the wireframe

---

## Step 2 — Geometry Extraction (`src/step_geometry.rs`)

### 2a. Collect all edge curves

Rather than walking the full assembly hierarchy (which requires resolving
`REPRESENTATION_RELATIONSHIP_WITH_TRANSFORMATION` chains), we take a simpler approach
**sufficient for this one file**: directly iterate all `ManifoldSolidBrep` entities and
tessellate their edges. Each part's geometry is already in the global assembly frame
because AP214's `ITEM_DEFINED_TRANSFORMATION` maps the local origin (#61) to the placed
origin. We resolve that transform per BREP.

### 2b. Assembly transform resolution

Each BREP is linked to a placement via:

```
ITEM_DEFINED_TRANSFORMATION('IDTn','', #origin_placement, #part_placement)
```

`#part_placement` is an `AXIS2_PLACEMENT_3D(location, z_axis, x_axis)`.
This defines a 4×4 rigid-body matrix:

```
x_col = ref_dir (x axis)
y_col = z_axis × x_col  (right-hand y axis)
z_col = z_axis
t_col = location
```

We apply this matrix to every vertex before projection. If we cannot resolve a transform
for a BREP, we use the identity (renders in local space, still useful).

### 2c. Tessellate edges

For each `EDGE_CURVE { start, end, geom }`:

**LINE geometry:**
- Resolve `start` → `VertexPoint` → `CartesianPoint` → `[f64; 3]`
- Resolve `end`   → same
- Emit one segment: `[p_start, p_end]`

**CIRCLE geometry:**
- Resolve `Circle { placement, radius }`
- Resolve `Axis2Placement3D { location, axis, ref_dir }`
  - `center = CartesianPoint` at `location`
  - `z_hat = Direction` at `axis` (normal to circle plane)
  - `x_hat = Direction` at `ref_dir` (0° reference)
  - `y_hat = z_hat × x_hat`
- Get start and end points from `VertexPoint`s, project into the circle's local 2D
  frame to find `θ_start` and `θ_end` via `atan2`
- For a full circle (`start == end`): θ goes 0 → 2π
- Subdivide arc into **32 segments**; each sample:
  `P(θ) = center + radius*(cos θ * x_hat + sin θ * y_hat)`
- Emit 32 consecutive `[p_i, p_{i+1}]` segment pairs

Output: `Vec<[[f32; 3]; 2]>` — all line segments in 3D world space.

---

## Step 3 — Orthographic Projection

**Front elevation** = project onto XZ plane (drop Y):

```
screen_x = world_x
screen_y = world_z   (Z is the vertical axis in this model)
```

1. Compute bounding box over all projected points: `[x_min, x_max]`, `[z_min, z_max]`
2. Compute uniform scale to fit within NDC `[-0.9, 0.9]` on the larger axis:
   ```
   cx = (x_min + x_max) / 2.0
   cz = (z_min + z_max) / 2.0
   span = max(x_max - x_min, z_max - z_min)
   scale = 1.8 / span
   ndc_x = (x - cx) * scale
   ndc_y = (z - cz) * scale
   ```
3. Flatten into `Vec<[f32; 2]>` — pairs of 2D NDC endpoints ready for the VBO.

---

## Step 4 — OpenGL Rendering (`src/main.rs`)

Replace the triangle boilerplate:

### Shaders

```glsl
// vertex
#version 330 core
layout (location = 0) in vec2 pos;
void main() {
    gl_Position = vec4(pos, 0.0, 1.0);
}

// fragment
#version 330 core
out vec4 color;
void main() {
    color = vec4(0.9, 0.9, 0.9, 1.0);  // near-white lines
}
```

### Upload & draw

```rust
// At startup (after GL load):
let segments = step_geometry::load("as1-ac-214.stp");   // returns Vec<[f32; 2]>
let vertex_count = segments.len() as i32;

let vao = VertexArray::new().unwrap();
vao.bind();
let vbo = Buffer::new().unwrap();
vbo.bind(BufferType::Array);
buffer_data(BufferType::Array, bytemuck::cast_slice(&segments), GL_STATIC_DRAW);

unsafe {
    glVertexAttribPointer(0, 2, GL_FLOAT, GL_FALSE,
        (2 * size_of::<f32>()) as i32, 0 as *const _);
    glEnableVertexAttribArray(0);
}
clear_color(0.1, 0.1, 0.15, 1.0);   // dark blue-grey background

// In render loop (no uniforms needed):
unsafe {
    glClear(GL_COLOR_BUFFER_BIT);
    glDrawArrays(GL_LINES, 0, vertex_count);
}
```

Window title: `"STEP Viewer — as1-ac-214"`.

---

## Step 5 — Cargo.toml

No new dependencies required. The existing crates (`ogl33`, `beryllium`, `bytemuck`) cover
all rendering needs. The parser uses only `std`.

---

## Out of Scope for MVP

- Surface tessellation / filled polygons
- Hidden-line removal
- Camera controls or zoom
- Assembly tree / part selection
- Multiple views (plan, section, isometric)
- Curved surface tessellation (`CYLINDRICAL_SURFACE`, `CONICAL_SURFACE` faces — only
  their boundary **edges** are rendered)
- Error UI (panics on malformed input are acceptable)

---

## Acceptance Criteria

Running `cargo run` opens an 800×600 window showing recognisable wireframe outlines of the
AS1 assembly plate, L-brackets, and bolt holes projected onto the front elevation. Closing
the window exits cleanly.

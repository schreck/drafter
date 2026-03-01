mod gl_utils;
mod step_geometry;
mod step_parser;

use gl_utils::{buffer_data, clear_color, Buffer, BufferType, ShaderProgram, VertexArray};

use beryllium::{
    ContextFlag, Event, GlProfile, InitFlags, SdlGlAttr, SwapInterval, WindowFlags,
    WindowPosition, SDL,
};
use core::mem::size_of;
use ogl33::*;

const WIN_W: u32 = 800;
const WIN_H: u32 = 600;

const VERT_SHADER: &str = r#"#version 330 core
  layout (location = 0) in vec2 pos;
  void main() {
    gl_Position = vec4(pos, 0.0, 1.0);
  }
"#;

const FRAG_SHADER: &str = r#"#version 330 core
  out vec4 color;
  void main() {
    color = vec4(0.9, 0.9, 0.9, 1.0);
  }
"#;

/// Project 3-D segments onto the XZ plane (front elevation, looking along -Y),
/// then normalise so the geometry fits within NDC [-0.9, 0.9].
fn project_elevation(segments_3d: &[[f32; 3]]) -> Vec<[f32; 2]> {
    let mut xmin = f32::MAX;
    let mut xmax = f32::MIN;
    let mut zmin = f32::MAX;
    let mut zmax = f32::MIN;

    for v in segments_3d {
        xmin = xmin.min(v[0]);
        xmax = xmax.max(v[0]);
        zmin = zmin.min(v[2]);
        zmax = zmax.max(v[2]);
    }

    let cx = (xmin + xmax) / 2.0;
    let cz = (zmin + zmax) / 2.0;
    let span = (xmax - xmin).max(zmax - zmin);
    let scale = if span > 0.0 { 1.8 / span } else { 1.0 };

    segments_3d.iter().map(|v| [(v[0] - cx) * scale, (v[2] - cz) * scale]).collect()
}

fn main() {
    let step_file = std::env::args().nth(1).unwrap_or_else(|| {
        eprintln!("Usage: drafter <file.stp>");
        std::process::exit(1);
    });

    // ── Load and tessellate STEP geometry ────────────────────────────────────
    let entities = step_parser::parse(&step_file);
    let segments_3d = step_geometry::extract_segments(&entities);
    let vertices: Vec<[f32; 2]> = project_elevation(&segments_3d);
    let vertex_count = vertices.len() as i32;

    println!("Loaded {} line-segment vertices from {step_file}", vertex_count);

    // ── SDL + OpenGL context ─────────────────────────────────────────────────
    let sdl = SDL::init(InitFlags::Everything).expect("SDL init failed");

    sdl.gl_set_attribute(SdlGlAttr::MajorVersion, 3).unwrap();
    sdl.gl_set_attribute(SdlGlAttr::MinorVersion, 3).unwrap();
    sdl.gl_set_attribute(SdlGlAttr::Profile, GlProfile::Core).unwrap();

    let mut flags = 0_i32;
    if cfg!(target_os = "macos") {
        flags |= ContextFlag::ForwardCompatible;
    }
    if cfg!(debug_assertions) {
        flags |= ContextFlag::Debug;
    }
    sdl.gl_set_attribute(SdlGlAttr::Flags, flags).unwrap();

    let win = sdl
        .create_gl_window(
            &format!("STEP Viewer — {step_file}"),
            WindowPosition::Centered,
            WIN_W,
            WIN_H,
            WindowFlags::Shown,
        )
        .expect("couldn't create window");

    win.set_swap_interval(SwapInterval::Vsync);

    unsafe { load_gl_with(|f_name| win.get_proc_address(f_name.cast())) };

    // ── Upload geometry ───────────────────────────────────────────────────────
    clear_color(0.1, 0.1, 0.15, 1.0);

    let vao = VertexArray::new().expect("Couldn't make VAO");
    vao.bind();

    let vbo = Buffer::new().expect("Couldn't make VBO");
    vbo.bind(BufferType::Array);
    buffer_data(BufferType::Array, bytemuck::cast_slice(&vertices), GL_STATIC_DRAW);

    unsafe {
        glVertexAttribPointer(
            0,
            2,
            GL_FLOAT,
            GL_FALSE,
            size_of::<[f32; 2]>().try_into().unwrap(),
            0 as *const _,
        );
        glEnableVertexAttribArray(0);
    }

    let shader = ShaderProgram::from_vert_frag(VERT_SHADER, FRAG_SHADER).unwrap();
    shader.use_program();

    // ── Render loop ───────────────────────────────────────────────────────────
    'main_loop: loop {
        while let Some(event) = sdl.poll_events().and_then(Result::ok) {
            if let Event::Quit(_) = event {
                break 'main_loop;
            }
        }

        unsafe {
            glClear(GL_COLOR_BUFFER_BIT);
            glDrawArrays(GL_LINES, 0, vertex_count);
        }
        win.swap_window();
    }
}

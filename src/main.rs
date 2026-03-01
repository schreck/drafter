mod gl_utils;
use gl_utils::{buffer_data, clear_color, Buffer, BufferType, ShaderProgram, VertexArray};

use beryllium::{
    ContextFlag, Event, GlProfile, InitFlags, SdlGlAttr, SwapInterval, WindowFlags,
    WindowPosition, SDL,
};
use core::mem::size_of;
use ogl33::*;

type Vertex = [f32; 3];

const VERTICES: [Vertex; 3] =
    [[-0.5, -0.5, 0.0], [0.5, -0.5, 0.0], [0.0, 0.5, 0.0]];

const VERT_SHADER: &str = r#"#version 330 core
  layout (location = 0) in vec3 pos;
  void main() {
    gl_Position = vec4(pos.x, pos.y, pos.z, 1.0);
  }
"#;

const FRAG_SHADER: &str = r#"#version 330 core
  out vec4 final_color;
  void main() {
    final_color = vec4(1.0, 0.5, 0.2, 1.0);
  }
"#;

fn main() {
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
        .create_gl_window("Triangle", WindowPosition::Centered, 800, 600, WindowFlags::Shown)
        .expect("couldn't make a window and context");

    win.set_swap_interval(SwapInterval::Vsync);

    unsafe { load_gl_with(|f_name| win.get_proc_address(f_name.cast())) };

    clear_color(0.2, 0.3, 0.3, 1.0);

    let vao = VertexArray::new().expect("Couldn't make a VAO");
    vao.bind();

    let vbo = Buffer::new().expect("Couldn't make a VBO");
    vbo.bind(BufferType::Array);
    buffer_data(BufferType::Array, bytemuck::cast_slice(&VERTICES), GL_STATIC_DRAW);

    unsafe {
        glVertexAttribPointer(
            0,
            3,
            GL_FLOAT,
            GL_FALSE,
            size_of::<Vertex>().try_into().unwrap(),
            0 as *const _,
        );
        glEnableVertexAttribArray(0);
    }

    let shader_program =
        ShaderProgram::from_vert_frag(VERT_SHADER, FRAG_SHADER).unwrap();
    shader_program.use_program();

    'main_loop: loop {
        while let Some(event) = sdl.poll_events().and_then(Result::ok) {
            if matches!(event, Event::Quit(_)) {
                break 'main_loop;
            }
        }

        unsafe {
            glClear(GL_COLOR_BUFFER_BIT);
            glDrawArrays(GL_TRIANGLES, 0, 3);
        }
        win.swap_window();
    }
}

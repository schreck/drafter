mod gl_utils;
use gl_utils::{
    buffer_data, clear_color, Buffer, BufferType, ShaderProgram, VertexArray,
};

use beryllium::{
    ContextFlag, Event, GlProfile, InitFlags, SdlGlAttr, SwapInterval, WindowFlags,
    WindowPosition, SDL,
};
use core::mem::size_of;
use ogl33::*;

const WIN_W: f32 = 800.0;
const WIN_H: f32 = 600.0;

type Vertex = [f32; 3];

// Small triangle centered at origin; the shader shifts it with a uniform
const VERTICES: [Vertex; 3] =
    [[0.0, 0.04, 0.0], [-0.03, -0.04, 0.0], [0.03, -0.04, 0.0]];

const VERT_SHADER: &str = r#"#version 330 core
  layout (location = 0) in vec3 pos;
  uniform vec2 offset;
  void main() {
    gl_Position = vec4(pos.x + offset.x, pos.y + offset.y, pos.z, 1.0);
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
        .create_gl_window("Mouse Triangle", WindowPosition::Centered, WIN_W as u32, WIN_H as u32, WindowFlags::Shown)
        .expect("couldn't make a window and context");

    win.set_swap_interval(SwapInterval::Vsync);

    unsafe { load_gl_with(|f_name| win.get_proc_address(f_name.cast())) };

    clear_color(0.2, 0.3, 0.3, 1.0);

    let vao = VertexArray::new().expect("Couldn't make a VAO");
    vao.bind();

    let vbo = Buffer::new().expect("Couldn't make the vertex buffer");
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

    let offset_location = unsafe {
        glGetUniformLocation(shader_program.0, b"offset\0".as_ptr().cast())
    };

    let mut mouse_x: f32 = WIN_W / 2.0;
    let mut mouse_y: f32 = WIN_H / 2.0;

    'main_loop: loop {
        while let Some(event) = sdl.poll_events().and_then(Result::ok) {
            match event {
                Event::Quit(_) => break 'main_loop,
                Event::MouseMotion(e) => {
                    mouse_x = e.x_pos as f32;
                    mouse_y = e.y_pos as f32;
                }
                _ => {}
            }
        }

        // Convert screen coords to NDC: x in [-1,1], y flipped
        let ndc_x = (mouse_x / WIN_W) * 2.0 - 1.0;
        let ndc_y = 1.0 - (mouse_y / WIN_H) * 2.0;

        unsafe {
            glUniform2f(offset_location, ndc_x, ndc_y);
            glClear(GL_COLOR_BUFFER_BIT);
            glDrawArrays(GL_TRIANGLES, 0, 3);
        }
        win.swap_window();
    }
}

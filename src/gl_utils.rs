use core::convert::TryInto;
use ogl33::*;

pub fn clear_color(r: f32, g: f32, b: f32, a: f32) {
    unsafe { glClearColor(r, g, b, a) }
}

pub fn buffer_data(ty: BufferType, data: &[u8], usage: GLenum) {
    unsafe {
        glBufferData(
            ty as GLenum,
            data.len().try_into().unwrap(),
            data.as_ptr().cast(),
            usage,
        );
    }
}

pub struct VertexArray(pub GLuint);
impl VertexArray {
    pub fn new() -> Option<Self> {
        let mut vao = 0;
        unsafe { glGenVertexArrays(1, &mut vao) };
        if vao != 0 { Some(Self(vao)) } else { None }
    }

    pub fn bind(&self) {
        unsafe { glBindVertexArray(self.0) }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BufferType {
    Array = GL_ARRAY_BUFFER as isize,
    ElementArray = GL_ELEMENT_ARRAY_BUFFER as isize,
}

pub struct Buffer(pub GLuint);
impl Buffer {
    pub fn new() -> Option<Self> {
        let mut vbo = 0;
        unsafe { glGenBuffers(1, &mut vbo) };
        if vbo != 0 { Some(Self(vbo)) } else { None }
    }

    pub fn bind(&self, ty: BufferType) {
        unsafe { glBindBuffer(ty as GLenum, self.0) }
    }
}

pub enum ShaderType {
    Vertex = GL_VERTEX_SHADER as isize,
    Fragment = GL_FRAGMENT_SHADER as isize,
}

pub struct Shader(pub GLuint);
impl Shader {
    pub fn from_source(ty: ShaderType, source: &str) -> Result<Self, String> {
        let shader = unsafe { glCreateShader(ty as GLenum) };
        if shader == 0 {
            return Err("Couldn't allocate shader".to_string());
        }
        unsafe {
            glShaderSource(
                shader,
                1,
                &(source.as_bytes().as_ptr().cast()),
                &(source.len().try_into().unwrap()),
            );
            glCompileShader(shader);
        }
        let mut compiled = 0;
        unsafe { glGetShaderiv(shader, GL_COMPILE_STATUS, &mut compiled) };
        if compiled != i32::from(GL_TRUE) {
            let mut needed_len = 0;
            unsafe { glGetShaderiv(shader, GL_INFO_LOG_LENGTH, &mut needed_len) };
            let mut v: Vec<u8> = Vec::with_capacity(needed_len.try_into().unwrap());
            let mut len_written = 0_i32;
            unsafe {
                glGetShaderInfoLog(shader, v.capacity().try_into().unwrap(), &mut len_written, v.as_mut_ptr().cast());
                v.set_len(len_written.try_into().unwrap());
            }
            unsafe { glDeleteShader(shader) };
            return Err(String::from_utf8_lossy(&v).into_owned());
        }
        Ok(Self(shader))
    }

    pub fn delete(self) {
        unsafe { glDeleteShader(self.0) };
    }
}

pub enum PolygonMode {
    Point = GL_POINT as isize,
    Line = GL_LINE as isize,
    Fill = GL_FILL as isize,
}

pub fn polygon_mode(mode: PolygonMode) {
    unsafe { glPolygonMode(GL_FRONT_AND_BACK, mode as GLenum) };
}

pub struct ShaderProgram(pub GLuint);
impl ShaderProgram {
    pub fn from_vert_frag(vert: &str, frag: &str) -> Result<Self, String> {
        let prog = unsafe { glCreateProgram() };
        if prog == 0 {
            return Err("Couldn't allocate program".to_string());
        }
        let v = Shader::from_source(ShaderType::Vertex, vert)
            .map_err(|e| format!("Vertex Compile Error: {}", e))?;
        let f = Shader::from_source(ShaderType::Fragment, frag)
            .map_err(|e| format!("Fragment Compile Error: {}", e))?;
        unsafe {
            glAttachShader(prog, v.0);
            glAttachShader(prog, f.0);
            glLinkProgram(prog);
        }
        v.delete();
        f.delete();
        let mut success = 0;
        unsafe { glGetProgramiv(prog, GL_LINK_STATUS, &mut success) };
        if success != i32::from(GL_TRUE) {
            let mut needed_len = 0;
            unsafe { glGetProgramiv(prog, GL_INFO_LOG_LENGTH, &mut needed_len) };
            let mut v: Vec<u8> = Vec::with_capacity(needed_len.try_into().unwrap());
            let mut len_written = 0_i32;
            unsafe {
                glGetProgramInfoLog(prog, v.capacity().try_into().unwrap(), &mut len_written, v.as_mut_ptr().cast());
                v.set_len(len_written.try_into().unwrap());
            }
            unsafe { glDeleteProgram(prog) };
            return Err(format!("Program Link Error: {}", String::from_utf8_lossy(&v)));
        }
        Ok(Self(prog))
    }

    pub fn use_program(&self) {
        unsafe { glUseProgram(self.0) };
    }
}

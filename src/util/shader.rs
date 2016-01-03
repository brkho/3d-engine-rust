// Utility module that allows for compiling shaders and linking OpenGL programs.
// TODO: Modify the gl-rs example shader compilation functions to have better error handling.
//
// Brian Ho
// brian@brkho.com

extern crate gl;
extern crate time;

use self::gl::types::*;
use std::ptr;
use std::str;
use std::ffi::CString;
use std::fs::File;
use std::io::Read;


// Compile the shader given a path to an external GLSL file. This is mostly
// pulled from the triangle.rs example from the gl-rs repo.
pub fn compile_shader(path: &str, ty: GLenum) -> GLuint {
    // Read in the external file and use its contents as the source.
    let mut fd = File::open(path).unwrap();
    let mut src = String::new();
    fd.read_to_string(&mut src).unwrap();

    let shader;
    unsafe {
        shader = gl::CreateShader(ty);
        // Attempt to compile the shader.
        let c_str = CString::new(src.as_bytes()).unwrap();
        gl::ShaderSource(shader, 1, &c_str.as_ptr(), ptr::null());
        gl::CompileShader(shader);

        // See if the shader compilation failed.
        let mut status = gl::FALSE as GLint;
        gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut status);

        // If the compilation failed, panic and output the error.
        if status != (gl::TRUE as GLint) {
            let mut len = 0;
            gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut len);
            let mut buf = Vec::with_capacity(len as usize);
            // Skip the trailing null character.
            buf.set_len((len as usize) - 1);
            gl::GetShaderInfoLog(
                    shader, len, ptr::null_mut(), buf.as_mut_ptr() as *mut GLchar);
            panic!("{}", str::from_utf8(&buf).ok().expect(
                    "ShaderInfoLog not valid utf8"));
        }
    }
    shader
}

// Link the program given a vertex shader and a fragment shader. This is
// entirely copied off the triangle.rs example from the gl-rs repo.
pub fn link_program(vs: GLuint, fs: GLuint) -> GLuint { unsafe {
    let program = gl::CreateProgram();
    gl::AttachShader(program, vs);
    gl::AttachShader(program, fs);
    gl::LinkProgram(program);
    // See if the shader compilation failed.
    let mut status = gl::FALSE as GLint;
    gl::GetProgramiv(program, gl::LINK_STATUS, &mut status);

    // If the compilation failed, panic and output the error.
    if status != (gl::TRUE as GLint) {
        let mut len: GLint = 0;
        gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut len);
        let mut buf = Vec::with_capacity(len as usize);
        // Skip the trailing null character.
        buf.set_len((len as usize) - 1);
        gl::GetProgramInfoLog(
                program, len, ptr::null_mut(), buf.as_mut_ptr() as *mut GLchar);
        panic!("{}", str::from_utf8(&buf).ok().expect(
                "ProgramInfoLog not valid utf8"));
    }
    program
} }

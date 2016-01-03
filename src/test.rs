// Work in progress of a 2D game using glutin for context creation/input
// handling and gl-rs for OpenGL bindings. The game will be a simple top down
// action-RPG created for educational purposes to assess the viability of Rust
// as a video game development language.
//
// Brian Ho
// brian@brkho.com

#[macro_use]
extern crate mmo;
extern crate cgmath;
extern crate glutin;
extern crate gl;
extern crate time;

use glutin::{Event, Window};

use cgmath::*;
use gl::types::*;
use std::mem;
use std::ptr;
use std::str;
use std::ffi::CString;
use std::fs::File;
use std::io::Read;
use std::process;
use mmo::util::bmp;

// Redeclaration of the constant void pointer type for ease of use.
type CVoid = *const std::os::raw::c_void;

// Compile the shader given a path to an external GLSL file. This is mostly
// pulled from the triangle.rs example from the gl-rs repo.
fn compile_shader(path: &str, ty: GLenum) -> GLuint {
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
fn link_program(vs: GLuint, fs: GLuint) -> GLuint { unsafe {
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

// Main loop for the game.
fn main() {
    let vertices: Vec<GLfloat> = vec![
        -0.5, -0.5, -0.5, 0.0, 0.0,
         0.5, -0.5, -0.5, 1.0, 0.0,
         0.5,  0.5, -0.5, 1.0, 1.0,
         0.5,  0.5, -0.5, 1.0, 1.0,
        -0.5,  0.5, -0.5, 0.0, 1.0,
        -0.5, -0.5, -0.5, 0.0, 0.0,

        -0.5, -0.5,  0.5, 0.0, 0.0,
         0.5, -0.5,  0.5, 1.0, 0.0,
         0.5,  0.5,  0.5, 1.0, 1.0,
         0.5,  0.5,  0.5, 1.0, 1.0,
        -0.5,  0.5,  0.5, 0.0, 1.0,
        -0.5, -0.5,  0.5, 0.0, 0.0,

        -0.5,  0.5,  0.5, 1.0, 0.0,
        -0.5,  0.5, -0.5, 1.0, 1.0,
        -0.5, -0.5, -0.5, 0.0, 1.0,
        -0.5, -0.5, -0.5, 0.0, 1.0,
        -0.5, -0.5,  0.5, 0.0, 0.0,
        -0.5,  0.5,  0.5, 1.0, 0.0,

         0.5,  0.5,  0.5, 1.0, 0.0,
         0.5,  0.5, -0.5, 1.0, 1.0,
         0.5, -0.5, -0.5, 0.0, 1.0,
         0.5, -0.5, -0.5, 0.0, 1.0,
         0.5, -0.5,  0.5, 0.0, 0.0,
         0.5,  0.5,  0.5, 1.0, 0.0,

        -0.5, -0.5, -0.5, 0.0, 1.0,
         0.5, -0.5, -0.5, 1.0, 1.0,
         0.5, -0.5,  0.5, 1.0, 0.0,
         0.5, -0.5,  0.5, 1.0, 0.0,
        -0.5, -0.5,  0.5, 0.0, 0.0,
        -0.5, -0.5, -0.5, 0.0, 1.0,

        -0.5,  0.5, -0.5, 0.0, 1.0,
         0.5,  0.5, -0.5, 1.0, 1.0,
         0.5,  0.5,  0.5, 1.0, 0.0,
         0.5,  0.5,  0.5, 1.0, 0.0,
        -0.5,  0.5,  0.5, 0.0, 0.0,
        -0.5,  0.5, -0.5, 0.0, 1.0
    ];


    // let vertices: Vec<GLfloat> = vec![
    //     -0.5,  0.5, 0.0, 0.0, // Top-left
    //      0.5,  0.5, 1.0, 0.0, // Top-right
    //      0.5, -0.5, 1.0, 1.0, // Bottom-right
    //     -0.5, -0.5, 0.0, 1.0, // Bottom-left
    // ];

    // let elements: Vec<GLuint> = vec![
    //     0, 1, 2,
    //     2, 3, 0
    // ];

    // Create the window. Should be using a builder here, but whatever.
    let window = Window::new().unwrap();
    unsafe { window.make_current().unwrap() };
    window.set_title("OpenGL Shenanigans");

    // Some magic OpenGL loading with similarly magical closures.
    gl::load_with(|symbol| window.get_proc_address(symbol) as *const _);

    // Load in the shaders and link the program.
    let vs = compile_shader("std.vert", gl::VERTEX_SHADER);
    let fs = compile_shader("std.frag", gl::FRAGMENT_SHADER);
    let program = link_program(vs, fs);
    let mut vao = 0;
    let mut vbo = 0;
    // let mut ebo = 0;
    let mut textures = vec![0 as u32; 2];

    unsafe {
        // Create Vertex Array Object.
        gl::GenVertexArrays(1, &mut vao);
        gl::BindVertexArray(vao);

        // Create a Vertex Buffer Object and copy the vertex data to it.
        gl::GenBuffers(1, &mut vbo);
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::BufferData(
                gl::ARRAY_BUFFER, float_size!(vertices.len(), GLsizeiptr),
                vec_to_addr!(vertices), gl::STATIC_DRAW);
        gl::Enable(gl::DEPTH_TEST);

        // Create Texture Object.
        gl::GenTextures(2, textures.get_unchecked_mut(0) as *mut u32);

        gl::ActiveTexture(gl::TEXTURE0);
        gl::BindTexture(gl::TEXTURE_2D, textures[0]);
        let brian_tex = bmp::decode_bmp("brian.bmp").unwrap();
        let brian_img = brian_tex.get_rgb_vec();
        gl::TexImage2D(
                gl::TEXTURE_2D, 0, gl::RGB as GLsizei, brian_tex.width as GLsizei,
                brian_tex.height as GLint, 0, gl::RGB as GLuint, gl::UNSIGNED_BYTE,
                vec_to_addr!(brian_img));
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as GLint);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as GLint);
        gl::GenerateMipmap(gl::TEXTURE_2D);
        gl::TexParameteri(
                gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER,
                gl::LINEAR_MIPMAP_NEAREST as GLint);
        gl::TexParameteri(
                gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER,
                gl::LINEAR_MIPMAP_NEAREST as GLint);

        gl::ActiveTexture(gl::TEXTURE1);
        gl::BindTexture(gl::TEXTURE_2D, textures[1]);
        let samantha_tex = bmp::decode_bmp("samantha.bmp").unwrap();
        let samantha_img = samantha_tex.get_rgb_vec();
        gl::TexImage2D(
                gl::TEXTURE_2D, 0, gl::RGB as GLsizei, samantha_tex.width as GLsizei,
                samantha_tex.height as GLint, 0, gl::RGB as GLuint, gl::UNSIGNED_BYTE,
                vec_to_addr!(samantha_img));
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::MIRRORED_REPEAT as GLint);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::MIRRORED_REPEAT as GLint);
        gl::GenerateMipmap(gl::TEXTURE_2D);
        gl::TexParameteri(
                gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER,
                gl::LINEAR_MIPMAP_NEAREST as GLint);
        gl::TexParameteri(
                gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER,
                gl::LINEAR_MIPMAP_NEAREST as GLint);

        // Use shader program
        gl::UseProgram(program);
        gl::BindFragDataLocation(program, 0, gl_str!("out_color"));

        // Specify the layout of the vertex data
        let pos_attr = gl::GetAttribLocation(program, gl_str!("position"));
        gl::EnableVertexAttribArray(pos_attr as GLuint);
        gl::VertexAttribPointer(
                pos_attr as GLuint, 3, gl::FLOAT, gl::FALSE as GLboolean,
                float_size!(5, GLsizei), ptr::null());

        let tex_attr = gl::GetAttribLocation(program, gl_str!("texcoord"));
        gl::EnableVertexAttribArray(tex_attr as GLuint);
        gl::VertexAttribPointer(
                tex_attr as GLuint, 2, gl::FLOAT, gl::FALSE as GLboolean,
                float_size!(5, GLsizei), float_size!(3, CVoid));

        // gl::GenBuffers(1, &mut ebo);
        // gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
        // gl::BufferData(
        //         gl::ELEMENT_ARRAY_BUFFER, float_size!(elements.len(), GLsizeiptr),
        //         vec_to_addr!(elements), gl::STATIC_DRAW);

        gl::Uniform1i(gl::GetUniformLocation(program, gl_str!("brian_tex")), 0);
        gl::Uniform1i(gl::GetUniformLocation(program, gl_str!("samantha_tex")), 1);
    }

    let mut last_time = time::now().to_timespec();
    let start_time = time::now().to_timespec();
    let time_location = unsafe { gl::GetUniformLocation(program, gl_str!("elapsed")) };
    let model_location = unsafe { gl::GetUniformLocation(program, gl_str!("model")) };
    let mut model = Decomposed {
            scale: 1.0,
            rot: Quaternion::from_axis_angle(vec3(0.0, 0.0, 1.0), Rad::from(deg(0.0))),
            disp: vec3(0.0, 0.0, 0.0) };
    let mut view = Matrix4::look_at(
            Point3::new(1.2, 1.2, 1.4),
            Point3::new(0.0, 0.0, 0.0),
            vec3(0.0, 0.0, 1.0));
    unsafe {
        let view_location = gl::GetUniformLocation(program, gl_str!("view"));
        gl::UniformMatrix4fv(
                view_location, 1, gl::FALSE as GLboolean,
                view.as_mut_ptr());
    }

    let (w_width, w_height) = window.get_outer_size().unwrap();
    let proj = PerspectiveFov {
            fovy: Rad::from(deg(45.0)),
            aspect: w_width as f32 / w_height as f32,
            near: 1.0,
            far: 10.0 };
    unsafe {
        let proj_loaction = gl::GetUniformLocation(program, gl_str!("proj"));
        gl::UniformMatrix4fv(
                proj_loaction, 1, gl::FALSE as GLboolean,
                Matrix4::from(proj).as_mut_ptr());
    }

    // Constant 60 FPS target frame rate.
    let mut msec_remaining = 1000.0 / 60.0;
    loop {
        // Get elapsed time since last update in ms.
        let curr_time = time::now().to_timespec();
        let elapsed_msec = (time::now().to_timespec() - start_time).num_milliseconds();
        let elapsed_sec = elapsed_msec as f32 / 1000.0;
        let delta = (curr_time - last_time).num_milliseconds() as f32 / 1000.0;
        msec_remaining -= delta;
        if msec_remaining > 0.0 {
            continue;
        }
        msec_remaining = 1000.0 / 60.0;

        unsafe {
            gl::Uniform1f(time_location, elapsed_sec);
            model.rot = Quaternion::from_axis_angle(
                    vec3(0.0, 0.0, 1.0), Rad::from(deg(elapsed_sec * 30.0)));
            gl::UniformMatrix4fv(
                    model_location, 1, gl::FALSE as GLboolean,
                    Matrix4::from(model).as_mut_ptr());
        }


        // println!("time elapsed: {}", delta);
        last_time = curr_time;

        // poll_events returns an iterator for Event which we match against.
        for event in window.poll_events() {
            match event {
                // Exit the entire program if the window closes.
                Event::Closed => process::exit(0),
                _ => ()
            }
        }

        unsafe {
            gl::ClearColor(0.3, 0.3, 0.3, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

            // Draw a triangle from the 3 vertices
            // gl::DrawElements(
                // gl::TRIANGLES, 6, gl::UNSIGNED_INT, 0 as CVoid);
            gl::DrawArrays(gl::TRIANGLES, 0, 36);
        }

        // We can update and draw here after we handle events and swap buffers.
        window.swap_buffers().unwrap();
    }
}
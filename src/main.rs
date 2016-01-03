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
// extern crate time;

use cgmath::*;
use gl::types::*;
use glutin::{Window, Event};
use std::mem;
use std::ptr;
use std::ffi::CString;
use std::process;
use mmo::util::shader;
use std::rc::Rc;

// Redeclaration of the constant void pointer type for ease of use.
type CVoid = *const std::os::raw::c_void;

// Aliasing of cgmath types for uniformity in the game engine.
pub type Vector3D = Vector3<GLfloat>;
pub type Vector4D = Vector4<GLfloat>;

// Represents a color in RGBA with intensity values from 0.0 to 1.0.
pub struct Color {
    pub r: GLfloat,
    pub g: GLfloat,
    pub b: GLfloat,
    pub a: GLfloat,
}

impl Color {
    // Default constructor for RGBA Color structs.
    pub fn new(red: f32, green: f32, blue: f32, alpha: f32) -> Color {
        Color { r: red, g: green, b: blue, a: alpha }
    }

    // Alternative constructor for RGB Color structs with alpha set to 1.0.
    pub fn new_rgb(red: f32, green: f32, blue: f32) -> Color {
        Color { r: red, g: green, b: blue, a: 1.0 }
    }

    // Default constructor for RGBA Color structs for range 0-255.
    pub fn new_u8(red: u8, green: u8, blue: u8, alpha: u8) -> Color {
        Color {
                r: red as f32 / 255.0, g: green as f32 / 255.0,
                b: blue as f32 / 255.0, a: alpha as f32 / 255.0 }
    }

    // Alternative constructor for RGB Color structs with alpha set to 255.
    pub fn new_rgb_u8(red: u8, green: u8, blue: u8) -> Color {
        Color { r: red as f32 / 255.0, g: green as f32 / 255.0, b: blue as f32 / 255.0, a: 1.0 }
    }
}

// Currently unimplemented because I do not have a way of loading in COLLADA/fbx files.
pub struct Material { pub id: u8 }

impl Material {
    // Default constructor to be implemented later.
    pub fn new() -> Material {
        Material { id: 0 }
    }
}

// Stores information about the model which can be instantiated to create a ModelInstance. 
pub struct ModelInfo {
    pub vertices: Vec<GLfloat>,
    pub color: Color,
    pub mat: Material,
}

impl ModelInfo {
    // Default constructor with color initialized to <1.0, 1.0, 1.0, 1.0>.
    pub fn new(vertices: Vec<GLfloat>, mat: Material) -> ModelInfo {
        ModelInfo { vertices: vertices, color: Color::new_rgb(1.0, 1.0, 1.0), mat: mat }
    }

    // Constructor to create a ModelInfo with a Color.
    pub fn new_with_color(vertices: Vec<GLfloat>, color: Color, mat: Material) -> ModelInfo {
        ModelInfo { vertices: vertices, color: color, mat: mat }
    }
}

// An instantiazation of a ModelInfo that represents a model in-game. This has a variety of
// positional attributes used to render the instance.
pub struct ModelInstance {
    pub info: Rc<ModelInfo>,
    pub pos: Vector3D,
    pub scale: Vector3D,
    pub rot: Vector4D,
}

impl ModelInstance {
    // Create an instance from a reference counted pointer to a ModelInfo struct.
    pub fn from(info: Rc<ModelInfo>) -> ModelInstance {
        let pos = Vector3D::new(0.0, 0.0, 0.0);
        let scale = Vector3D::new(0.0, 0.0, 0.0);
        let rot = Vector4D::new(0.0, 0.0, 0.0, 0.0);
        ModelInstance { info: info, pos: pos, scale: scale, rot: rot }
    }
}

// Specifies two methods for getting the view and projection matrices.
pub trait Camera {
    fn get_view_matrix(&self) -> Matrix4<GLfloat>;
    fn get_projection_matrix(&self) -> Matrix4<GLfloat>;
}

// A representation of a camera with a perspective projection. This implements the Camera trait, so
// it can be used as a camera for rendering the game.
pub struct PerspectiveCamera {
    pub pos: Vector3D,
    pub target: Vector3D,
    proj: Matrix4<GLfloat>,
    up: Vector3D,
}

// Implementation of the Camera methods for PerspectiveCamera.
impl Camera for PerspectiveCamera {
    // Calculate the view matrix from the PerspectiveCamera's position and target.
    fn get_view_matrix(&self) -> Matrix4<GLfloat> {
        Matrix4::look_at(
                Point3::from_vec(self.pos),
                Point3::from_vec(self.target),
                self.up)
    }

    // Since we precompute the projection, we can just return it here.
    fn get_projection_matrix(&self) -> Matrix4<GLfloat> { self.proj }
}

// Implementation of PerspectiveCamera methods.
impl PerspectiveCamera {
    // Constructor to initialize the fields and set up the Projection matrix.
    pub fn new(pos: Vector3D, target: Vector3D, aspect: f32, fov: f32,
            near: f32, far: f32) -> PerspectiveCamera {
        let up = Vector3D::new(0.0, 0.0, 1.0);
        PerspectiveCamera::new_with_up(pos, target, up, aspect, fov, near, far)
    }

    // Constructor to initialize the fields and set up the Projection matrix with a specified up
    // vector.
    pub fn new_with_up(pos: Vector3D, target: Vector3D, up: Vector3D, aspect: f32,
            fov: f32, near: f32, far: f32) -> PerspectiveCamera {
        let proj = PerspectiveFov {
                fovy: Rad::from(deg(fov)),
                aspect: aspect,
                near: near,
                far: far };
        PerspectiveCamera { pos: pos, target: target, up: up, proj: Matrix4::from(proj) }
    }
}

// TODO: Write the OrthographicCamera.
// pub struct OrthographicCamera { }

// Light source that emanates from a fixed point with specified intensity and attenuation.
pub struct PointLight {
    pub intensity: Color,
    pub position: Vector3D,
    pub const_attn: f32,
    pub linear_attn: f32,
    pub quad_attn: f32,
}

// Light source that shines from an infinite distance from a direction (such as the sun).
pub struct DirectionalLight {
    pub intensity: Color,
    pub direction: Vector3D,
}

// Light source that emanates from a fixed point like a PointLight, but has a certain arc and
// falloff (like a flashlight).
pub struct SpotLight {
    pub intensity: Color,
    pub position: Vector3D,
    pub direction: Vector3D,
    pub const_attn: f32,
    pub linear_attn: f32,
    pub quad_attn: f32,
    pub cutoff: f32,
    pub dropoff: f32,
}

// A window for graphics drawing that is managed by the graphics module. This is a thin wrapper
// around the glutin Window class and will manage draws to the glutin window.
pub struct GameWindow {
    pub bg_color: Color,
    pub camera: Option<Box<Camera>>,
    gl_window: Window,
    point_lights: Vec<Option<PointLight>>,
    directional_lights: Vec<Option<DirectionalLight>>,
    spot_lights: Vec<Option<SpotLight>>,
    vao: GLuint,
    vbo: GLuint,
    program: GLuint,
}

impl GameWindow {
    // Initializes a GameWindow with a black background and no camera. Note that the GameWindow
    // creation can fail suchas unsupported OpenGL, so it returns a Result.
    pub fn new(width: u32, height: u32, title: String) -> Result<GameWindow, String> {
        let bg_color = Color::new_rgb(0.0, 0.0, 0.0);
        let pl: Vec<Option<PointLight>> = Vec::new();
        let dl: Vec<Option<DirectionalLight>> = Vec::new();
        let sl: Vec<Option<SpotLight>> = Vec::new();

        // TODO: Handle the actual error reporting of glutin and make this code less ugly.
        let creation_err = "Unable to create GameWindow.";
        let gl_window = try!(Window::new().map_err(|_| creation_err.to_string()));
        unsafe { try!(gl_window.make_current().map_err(|_| creation_err.to_string())) }
        gl_window.set_title(&title);
        gl::load_with(|symbol| gl_window.get_proc_address(symbol) as *const _);

        let mut window = GameWindow {
                bg_color: bg_color, camera: None, gl_window: gl_window, vao: 0, vbo: 0,
                program: 0, point_lights: pl, directional_lights: dl, spot_lights: sl };
        // Begin unsafe OpenGL shenanigans. Here, we compile and link the shaders, set up the VAO
        // and VBO, and specify the layout of the vertex data.
        unsafe {
            let vs = shader::compile_shader("std.vert", gl::VERTEX_SHADER);
            let fs = shader::compile_shader("std.frag", gl::FRAGMENT_SHADER);
            window.program = shader::link_program(vs, fs);

            gl::GenVertexArrays(1, &mut window.vao);
            gl::BindVertexArray(window.vao);
            gl::GenBuffers(1, &mut window.vbo);
            gl::BindBuffer(gl::ARRAY_BUFFER, window.vbo);
            gl::Enable(gl::DEPTH_TEST);

            gl::UseProgram(window.program);
            gl::BindFragDataLocation(window.program, 0, gl_str!("out_color"));

            let pos_attr = gl::GetAttribLocation(window.program, gl_str!("position"));
            gl::EnableVertexAttribArray(pos_attr as GLuint);
            gl::VertexAttribPointer(
                    pos_attr as GLuint, 3, gl::FLOAT, gl::FALSE as GLboolean,
                    float_size!(7, GLsizei), ptr::null());
            let color_attr = gl::GetAttribLocation(window.program, gl_str!("color"));
            gl::EnableVertexAttribArray(color_attr as GLuint);
            gl::VertexAttribPointer(
                    color_attr as GLuint, 4, gl::FLOAT, gl::FALSE as GLboolean,
                    float_size!(7, GLsizei), float_size!(3, CVoid));
        }

        window.set_size(width, height);
        window.clear();
        window.swap_buffers();
        Ok(window)
    }

    // Constructs and adds a PointLight to the scene. This then returns an u16 handle (internally
    // representing the index in the array) that can be used with the getter to modify light attrs.
    pub fn add_point_light(&mut self, intensity: Color, position: Vector3D, const_attn: f32,
            linear_attn: f32, quad_attn: f32) -> usize {
        let light = PointLight {
                intensity: intensity, position: position, const_attn: const_attn,
                linear_attn: linear_attn, quad_attn: quad_attn };
        GameWindow::add_light(light, &mut self.point_lights)
    }

    // Removes a PointLight from the scene given its handle.
    pub fn remove_point_light(&mut self, index: usize) {
        self.point_lights[index] = None;
    }

    // Gets a reference to a PointLight given its handle.
    pub fn get_point_light(&mut self, index: usize) -> &mut PointLight {
        (&mut self.point_lights[index]).as_mut().unwrap()
    }

    // Constructs and adds a DirectionalLight to the scene. This then returns an u16 handle
    // (internally representing the index in the array) that can be used with the getter to modify
    // light attrs.
    pub fn add_directional_light(&mut self, intensity: Color, direction: Vector3D) -> usize {
        let light = DirectionalLight { intensity: intensity, direction: direction };
        GameWindow::add_light(light, &mut self.directional_lights)
    }

    // Removes a DirectionalLight from the scene given its handle.
    pub fn remove_directional_light(&mut self, index: usize) {
        self.directional_lights[index] = None;
    }

    // Gets a reference to a DirectionalLight given its handle.
    pub fn get_directional_light(&mut self, index: usize) -> &mut DirectionalLight {
        (&mut self.directional_lights[index]).as_mut().unwrap()
    }

    // Constructs and adds a SpotLight to the scene. This then returns an u16 handle (internally
    // representing the index in the array) that can be used with the getter to modify light attrs.
    pub fn add_spot_light(&mut self, intensity: Color, position: Vector3D, direction: Vector3D,
            const_attn: f32, linear_attn: f32, quad_attn: f32, cutoff: f32,
            dropoff: f32) -> usize {
        let light = SpotLight {
                intensity: intensity, position: position, const_attn: const_attn,
                direction: direction, linear_attn: linear_attn, quad_attn: quad_attn,
                cutoff: cutoff, dropoff: dropoff };
        GameWindow::add_light(light, &mut self.spot_lights)
    }

    // Removes a SpotLight from the scene given its handle.
    pub fn remove_spot_light(&mut self, index: usize) {
        self.spot_lights[index] = None;
    }

    // Gets a reference to a SpotLight given its handle.
    pub fn get_spot_light(&mut self, index: usize) -> &mut SpotLight {
        (&mut self.spot_lights[index]).as_mut().unwrap()
    }

    // Helper function that adds a light to a specified vector of lights. This keeps track of
    // "holes" in the array and returns a handle to the first unused location in the array. If
    // there are no holes, then it adds the light to the end and returns the corresponding handle.
    fn add_light<T>(light: T, vector: &mut Vec<Option<T>>) -> usize {
        let mut index = None;
        for (i, elem) in vector.iter().enumerate() {
            match elem {
                &None => { index = Some(i); },
                &Some(_) => (),
            }
        }
        match index {
            None => {
                vector.push(Some(light));
                vector.len() - 1
            }
            Some(i) => {
                vector[i] = Some(light);
                i
            }
        }
    }

    // Sets the size of the window.
    pub fn set_size(&self, width: u32, height: u32) {
        self.gl_window.set_inner_size(width, height);
    }

    // Gets the gl_window.
    pub fn poll_events(&self) -> glutin::PollEventsIterator {
        self.gl_window.poll_events()
    }

    // Clears the screen and buffers.
    pub fn clear(&self) {
        unsafe {
            gl::ClearColor(self.bg_color.r, self.bg_color.g, self.bg_color.b, self.bg_color.a);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }
    }

    // Swaps the buffers.
    pub fn swap_buffers(&self) {
        self.gl_window.swap_buffers().unwrap();
    }
}

// Driver test program.
fn main() {
    let window = GameWindow::new(1024, 768, "Test Window".to_string()).unwrap();

    loop {
        for event in window.poll_events() {
            match event {
                Event::Closed => process::exit(0),
                _ => ()
            }
        }
    }
}
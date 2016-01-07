// Work in progress. This program still needs to be refactored into modules.
//
// Brian Ho
// brian@brkho.com

#[macro_use]
extern crate mmo;
extern crate cgmath;
extern crate glutin;
extern crate gl;
extern crate time;

use cgmath::{Point, Matrix, EuclideanVector};
use gl::types::*;
use glutin::{Window, Event, VirtualKeyCode, ElementState};
use mmo::util::shader;
use std::cell::Cell;
use std::ffi::CString;
use std::mem;
use std::process;
use std::ptr;
use std::rc::Rc;
use std::thread::sleep;
use std::time::Duration;

// Redeclaration of the constant void pointer type for ease of use.
type CVoid = *const std::os::raw::c_void;

// Aliasing of cgmath types for uniformity in the game engine.
pub type Vector3D = cgmath::Vector3<GLfloat>;
pub type Quaternion = cgmath::Quaternion<GLfloat>;

// Number of floats in a VBO.
const BUFFER_SIZE: usize = 65535;

// Contents of a VBO.
const VERTEX_POS_SIZE: usize = 3;
const VERTEX_COLOR_SIZE: usize = 4;
const VERTEX_SIZE: usize = VERTEX_POS_SIZE + VERTEX_COLOR_SIZE;

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

#[derive(Copy, Clone)]
pub struct BufferInfo {
    pub gen: usize,
    pub start: usize,
    pub size: usize,
    pub vbo: GLuint,
    pub vao: GLuint,
    pub ebo: Option<GLuint>,
}

// Stores information about the model which can be instantiated to create a ModelInstance. 
pub struct ModelInfo {
    pub vertices: Vec<GLfloat>,
    pub elements: Option<Vec<GLuint>>,
    pub color: Color,
    pub mat: Material,
    pub buffer_info: Cell<Option<BufferInfo>>,
}

impl ModelInfo {
    // Default constructor with color initialized to <1.0, 1.0, 1.0, 1.0>.
    pub fn new(vertices: Vec<GLfloat>, mat: Material) -> ModelInfo {
        ModelInfo::new_with_color(vertices, Color::new_rgb(1.0, 1.0, 1.0), mat)
    }

    // Constructor to create a ModelInfo with a Color.
    pub fn new_with_color(vertices: Vec<GLfloat>, color: Color, mat: Material) -> ModelInfo {
        ModelInfo { vertices: vertices, color: color, mat: mat, buffer_info: Cell::new(None),
                elements: None }
    }

    // Creates a box with specified size and color.
    pub fn new_box(scale_x: f32, scale_y: f32, scale_z: f32, color: Color) -> ModelInfo {
        let vertices: Vec<GLfloat> = vec![
            -0.5 * scale_x, -0.5 * scale_y, -0.5 * scale_z,
             0.5 * scale_x, -0.5 * scale_y, -0.5 * scale_z,
             0.5 * scale_x,  0.5 * scale_y, -0.5 * scale_z,
             0.5 * scale_x,  0.5 * scale_y, -0.5 * scale_z,
            -0.5 * scale_x,  0.5 * scale_y, -0.5 * scale_z,
            -0.5 * scale_x, -0.5 * scale_y, -0.5 * scale_z,

            -0.5 * scale_x, -0.5 * scale_y,  0.5 * scale_z,
             0.5 * scale_x, -0.5 * scale_y,  0.5 * scale_z,
             0.5 * scale_x,  0.5 * scale_y,  0.5 * scale_z,
             0.5 * scale_x,  0.5 * scale_y,  0.5 * scale_z,
            -0.5 * scale_x,  0.5 * scale_y,  0.5 * scale_z,
            -0.5 * scale_x, -0.5 * scale_y,  0.5 * scale_z,

            -0.5 * scale_x,  0.5 * scale_y,  0.5 * scale_z,
            -0.5 * scale_x,  0.5 * scale_y, -0.5 * scale_z,
            -0.5 * scale_x, -0.5 * scale_y, -0.5 * scale_z,
            -0.5 * scale_x, -0.5 * scale_y, -0.5 * scale_z,
            -0.5 * scale_x, -0.5 * scale_y,  0.5 * scale_z,
            -0.5 * scale_x,  0.5 * scale_y,  0.5 * scale_z,

             0.5 * scale_x,  0.5 * scale_y,  0.5 * scale_z,
             0.5 * scale_x,  0.5 * scale_y, -0.5 * scale_z,
             0.5 * scale_x, -0.5 * scale_y, -0.5 * scale_z,
             0.5 * scale_x, -0.5 * scale_y, -0.5 * scale_z,
             0.5 * scale_x, -0.5 * scale_y,  0.5 * scale_z,
             0.5 * scale_x,  0.5 * scale_y,  0.5 * scale_z,

            -0.5 * scale_x, -0.5 * scale_y, -0.5 * scale_z,
             0.5 * scale_x, -0.5 * scale_y, -0.5 * scale_z,
             0.5 * scale_x, -0.5 * scale_y,  0.5 * scale_z,
             0.5 * scale_x, -0.5 * scale_y,  0.5 * scale_z,
            -0.5 * scale_x, -0.5 * scale_y,  0.5 * scale_z,
            -0.5 * scale_x, -0.5 * scale_y, -0.5 * scale_z,

            -0.5 * scale_x,  0.5 * scale_y, -0.5 * scale_z,
             0.5 * scale_x,  0.5 * scale_y, -0.5 * scale_z,
             0.5 * scale_x,  0.5 * scale_y,  0.5 * scale_z,
             0.5 * scale_x,  0.5 * scale_y,  0.5 * scale_z,
            -0.5 * scale_x,  0.5 * scale_y,  0.5 * scale_z,
            -0.5 * scale_x,  0.5 * scale_y, -0.5 * scale_z
        ];
        ModelInfo::new_with_color(vertices, color, Material::new())
    }
}

// An instantiazation of a ModelInfo that represents a model in-game. This has a variety of
// positional attributes used to render the instance.
pub struct ModelInstance {
    pub info: Rc<ModelInfo>,
    pub pos: Vector3D,
    pub rot: Quaternion,
    pub scale: f32,
}

impl ModelInstance {
    // Create an instance from a reference counted pointer to a ModelInfo struct.
    pub fn from(info: Rc<ModelInfo>) -> ModelInstance {
        let pos = Vector3D::new(0.0, 0.0, 0.0);
        let rot = Quaternion::new(1.0, 0.0, 0.0, 0.0);
        let scale = 1.0;
        ModelInstance { info: info, pos: pos, scale: scale, rot: rot }
    }
}

// Specifies two methods for getting the view and projection matrices.
pub trait Camera {
    fn get_view_matrix(&self) -> cgmath::Matrix4<GLfloat>;
    fn get_projection_matrix(&self) -> cgmath::Matrix4<GLfloat>;
    fn get_fwd(&self) -> Vector3D;
    fn get_right(&self) -> Vector3D;
}

// A representation of a camera with a perspective projection. This implements the Camera trait, so
// it can be used as a camera for rendering the game.
pub struct PerspectiveCamera {
    pub pos: Vector3D,
    pub target: Vector3D,
    proj: cgmath::Matrix4<GLfloat>,
    up: Vector3D,
}

// Implementation of the Camera methods for PerspectiveCamera.
impl Camera for PerspectiveCamera {
    // Calculate the view matrix from the PerspectiveCamera's position and target.
    fn get_view_matrix(&self) -> cgmath::Matrix4<GLfloat> {
        cgmath::Matrix4::look_at(
                cgmath::Point3::from_vec(self.pos),
                cgmath::Point3::from_vec(self.target),
                self.up)
    }

    // Gets the forward vector from the view matrix.
    fn get_fwd(&self) -> Vector3D {
        let mat = self.get_view_matrix();
        Vector3D::new(mat.x[2], mat.y[2], mat.z[2])
    }

    // Gets the right vector from the view matrix.
    fn get_right(&self) -> Vector3D {
        let mat = self.get_view_matrix();
        Vector3D::new(mat.x[0], mat.y[0], mat.z[0])
    }

    // Since we precompute the projection, we can just return it here.
    fn get_projection_matrix(&self) -> cgmath::Matrix4<GLfloat> { self.proj }
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
        let proj = cgmath::PerspectiveFov {
                fovy: cgmath::Rad::from(cgmath::deg(fov)),
                aspect: aspect,
                near: near,
                far: far };
        PerspectiveCamera { pos: pos, target: target, up: up, proj: cgmath::Matrix4::from(proj) }
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
    pub cameras: Vec<Option<PerspectiveCamera>>,
    active_camera: Option<usize>,
    gl_window: Window,
    point_lights: Vec<Option<PointLight>>,
    directional_lights: Vec<Option<DirectionalLight>>,
    spot_lights: Vec<Option<SpotLight>>,
    program: GLuint,
    gen: usize,
    bound_vbo: Option<GLuint>,
    bound_ebo: Option<GLuint>,
    vbos: Vec<(GLuint, GLuint, usize)>, // (vao_id, vbo_id, size)
    ebos: Vec<(GLuint, usize)>, // (ebo_id, size)

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
                bg_color: bg_color, cameras: Vec::new(), gl_window: gl_window,
                program: 0, point_lights: pl, directional_lights: dl, spot_lights: sl,
                active_camera: None, gen: 0, vbos: Vec::new(), bound_vbo: None, ebos: Vec::new(),
                bound_ebo: None };
        // Begin unsafe OpenGL shenanigans. Here, we compile and link the shaders, set up the VAO
        // and VBO, and specify the layout of the vertex data.
        unsafe {
            let vs = shader::compile_shader("std.vert", gl::VERTEX_SHADER);
            let fs = shader::compile_shader("std.frag", gl::FRAGMENT_SHADER);
            window.program = shader::link_program(vs, fs);
            window.initialize_vbo();
            window.initialize_ebo();
            gl::Enable(gl::DEPTH_TEST);
            gl::UseProgram(window.program);
            gl::BindFragDataLocation(window.program, 0, gl_str!("out_color"));
        }

        window.set_size(width, height);
        window.clear();
        window.swap_buffers();
        Ok(window)
    }

    // A helper method for binding the VAO and VBO that sets/checks the previously bound buffer.
    fn bind_buffers_checked(&mut self, vao: Option<GLuint>, vbo: Option<GLuint>,
            ebo: Option<GLuint>) { unsafe {
        if match (self.bound_vbo, vbo) {
                (Some(vbo1), Some(vbo2)) => vbo1 != vbo2,
                (None, Some(_)) => true,
                (_, None) => false, } {
            match vao {
                Some(vao1) => { gl::BindVertexArray(vao1) },
                None => (),
            };
            self.bound_vbo = vbo;
            println!("Switching VBO buffers to {}...", vbo.unwrap());
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo.unwrap());
        }

        if match (self.bound_ebo, ebo) {
                (Some(ebo1), Some(ebo2)) => ebo1 != ebo2,
                (None, Some(_)) => true,
                (_, None) => false, } {
            self.bound_ebo = ebo;
            println!("Switching EBO buffers to {}...", ebo.unwrap());
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo.unwrap());
        }
    } }

    // Initializes a managed empty VBO of size BUFFER_SIZE and adds it to the vector of VBOs.
    fn initialize_vbo(&mut self) { unsafe {
        let mut vbo = 0;
        let mut vao = 0;
        gl::GenVertexArrays(1, &mut vao);
        gl::GenBuffers(1, &mut vbo);
        println!("generated VAO: {}, VBO: {}", vao, vbo);
        self.bind_buffers_checked(Some(vao), Some(vbo), None);
        gl::BufferData(
                gl::ARRAY_BUFFER, float_size!(BUFFER_SIZE, GLsizeiptr),
                0 as CVoid, gl::STATIC_DRAW);
        let pos_attr = gl::GetAttribLocation(self.program, gl_str!("position"));
        gl::EnableVertexAttribArray(pos_attr as GLuint);
        gl::VertexAttribPointer(
                pos_attr as GLuint, VERTEX_POS_SIZE as i32, gl::FLOAT, gl::FALSE as GLboolean,
                float_size!(VERTEX_SIZE, GLsizei), ptr::null());
        let color_attr = gl::GetAttribLocation(self.program, gl_str!("color"));
        gl::EnableVertexAttribArray(color_attr as GLuint);
        gl::VertexAttribPointer(
                color_attr as GLuint, VERTEX_COLOR_SIZE as i32, gl::FLOAT, gl::FALSE as GLboolean,
                float_size!(VERTEX_SIZE, GLsizei), float_size!(VERTEX_POS_SIZE, CVoid));
        self.vbos.push((vao, vbo, 0));
    }}

    // Initializes a managed empty EBO of size BUFFER_SIZE and adds it to the vector of EBOs.
    fn initialize_ebo(&mut self) { unsafe {
        let mut ebo = 0;
        gl::GenBuffers(1, &mut ebo);
        println!("generated EBO: {}", ebo);
        self.bind_buffers_checked(None, None, Some(ebo));
        gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER, uint_size!(BUFFER_SIZE, GLsizeiptr),
                0 as CVoid, gl::STATIC_DRAW);
        self.ebos.push((ebo, 0));
    }}


    // Adds a Camera to the engine and returns an integer handle to that camera that can be used
    // with get_camera() and detach_camera().
    pub fn attach_camera(&mut self, camera: PerspectiveCamera) -> usize {
        let mut index = None;
        for (i, elem) in self.cameras.iter().enumerate() {
            match elem {
                &None => { index = Some(i); },
                &Some(_) => (),
            }
        }
        match index {
            None => {
                self.cameras.push(Some(camera));
                self.cameras.len() - 1
            }
            Some(i) => {
                self.cameras[i] = Some(camera);
                i
            }
        }
    }

    // Removes a camera from the engine and returns a Result if it was successful.
    pub fn detach_camera(&mut self, handle: usize) -> Result<(), String> {
        if handle >= self.cameras.len() { return Err("Out of range.".to_string()); }
        match self.active_camera {
            None => (),
            Some(c) => if handle == c { self.active_camera = None; },
        }
        self.cameras[handle] = None;
        Ok(())
    }

    // Takes in a handle and returns a mut reference to the corresponding camera if it is within
    // range. Otherwise, return an Err.
    pub fn get_camera(&mut self, handle: usize) -> Result<&mut PerspectiveCamera, String> {
        if handle >= self.cameras.len() { return Err("Out of range.".to_string()); }
        Ok(self.cameras[handle].as_mut().unwrap())
    }

    // Gets a mut reference to the active camera. Returns Err if no current active camera.
    pub fn get_active_camera(&mut self) -> Result<&mut PerspectiveCamera, String> {
        match self.active_camera {
            None => Err("No currently active camera.".to_string()),
            Some(c) => self.get_camera(c)
        }
    }

    // Sets the active camera used for rendering given a handle.
    pub fn set_active_camera(&mut self, handle: usize) -> Result<(), String> {
        if handle >= self.cameras.len() { return Err("Out of range.".to_string()); }
        self.active_camera = Some(handle);
        Ok(())
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

    // Gets the size of the window in pixels. Again, just a poorly named wrapper. Sorry tomaka. :(
    pub fn get_size(&self) -> (u32, u32) {
        self.gl_window.get_inner_size_pixels().unwrap()
    }

    // Gets the aspect ratio of the window.
    pub fn get_aspect_ratio(&self) -> f32 {
        let (width, height) = self.get_size();
        (width as f32) / (height as f32)
    }

    // Maps/remaps a given Rc<ModelInfo> to VBO and EBO locations in the engine's managed buffers.
    pub fn map_vbo(&mut self, info: Rc<ModelInfo>) {
        let mut vertices: Vec<GLfloat> = Vec::new();
        for x in 0..info.vertices.len() {
            if x % 3 != 0 {
                continue;
            }
            vertices.push(info.vertices[x]);
            vertices.push(info.vertices[x + 1]);
            vertices.push(info.vertices[x + 2]);
            vertices.push(info.color.r);
            vertices.push(info.color.g);
            vertices.push(info.color.b);
            vertices.push(info.color.a);
        }
        // Find empty EBO space.
        let ebo_index = match info.elements {
            None => None,
            Some(ref elements) => {
                let mut index = None;
                for (i, ebo_pair) in self.ebos.iter().enumerate() {
                    if elements.len() < BUFFER_SIZE - ebo_pair.1 {
                        index = Some(i);
                        break;
                    }
                }
                match index {
                    None => {
                        self.initialize_ebo();
                        Some(self.ebos.len() - 1)
                    },
                    Some(_) => index,
                }
            }
        };

        // Find empty VBO space.
        let vbo_index = {
            let mut index = None;
            for (i, vbo_pair) in self.vbos.iter().enumerate() {
                if vertices.len() < BUFFER_SIZE - vbo_pair.2 {
                    index = Some(i);
                    break;
                }
            }
            match index {
                None => {
                    self.initialize_vbo();
                    self.vbos.len() - 1
                },
                Some(i) => i,
            }
        };

        let vbo_pair = self.vbos[vbo_index];
        let buffer_info = match ebo_index {
            None => BufferInfo {
                    start: vbo_pair.2 / VERTEX_SIZE, size: vertices.len() / VERTEX_SIZE,
                    gen: self.gen, vao: vbo_pair.0, vbo: vbo_pair.1, ebo: None },
            Some(i) => BufferInfo {
                    start: self.ebos[i].1, size: info.elements.as_ref().unwrap().len(),
                    gen: self.gen, vao: vbo_pair.0, vbo: vbo_pair.1, ebo: Some(self.ebos[i].0) },
        };
        self.bind_buffers_checked(Some(buffer_info.vao), Some(buffer_info.vbo), buffer_info.ebo);
        info.buffer_info.set(Some(buffer_info));

        if let Some(i) = ebo_index {
            let ebo_pair = self.ebos[i];
            let elements = info.elements.as_ref().unwrap();
            unsafe {
                gl::BufferSubData(
                        gl::ELEMENT_ARRAY_BUFFER, uint_size!(ebo_pair.1, GLintptr),
                        uint_size!(elements.len(), GLsizeiptr), vec_to_addr!(elements));
            }
            self.ebos[i] = (ebo_pair.0, ebo_pair.1 + elements.len());
        }

        unsafe {
            gl::BufferSubData(
                    gl::ARRAY_BUFFER, float_size!(vbo_pair.2, GLintptr),
                    float_size!(vertices.len(), GLsizeiptr), vec_to_addr!(vertices));
        }
        self.vbos[vbo_index] = (vbo_pair.0, vbo_pair.1, vbo_pair.2 + vertices.len());
    }

    // Clears the VBOs so that every ModelInfo currently mapped to the engine's VBO space must be
    // remapped on the next draw_instance() call.
    pub fn clear_vertex_buffers(&mut self) {
        self.gen += 1;
        for vbo_pair in self.vbos.iter_mut() {
            *vbo_pair = (vbo_pair.0, vbo_pair.1, 0);
        }
    }

    // Draw a ModelInstance to the window using a camera, position, vertices, and materials.
    // This method also manages the engine's VBO space and updates the BufferInfo of the instance's
    // ModelInfo. If there is no associated BufferInfo for a ModelInfo, then we find an empty space in
    // the engine's VBO space and assign a new BufferInfo. If there is no more empty space in any of
    // the managed VBOs, we create a new VBO and assign it there instead. There is also a
    // generation field in both the BufferInfo and the Engine. On clear_vertex_buffers(), we increment
    // this generation count in the engine. If the generation count on the ModelInfo does not match
    // the count of the Engine, we remap.
    pub fn draw_instance(&mut self, instance: &ModelInstance) {
        match instance.info.buffer_info.get() {
            None => { self.map_vbo(instance.info.clone()); },
            Some(i) => { if i.gen != self.gen { self.map_vbo(instance.info.clone()) }; },
        }

        let mut transform = {
            let camera = match self.active_camera {
                None => { return; },
                Some(c) => self.cameras[c].as_ref().unwrap(),
            };
            let model = cgmath::Matrix4::from(cgmath::Decomposed {
                    scale: instance.scale, rot: instance.rot, disp: instance.pos });
            let view = camera.get_view_matrix();
            let proj = camera.get_projection_matrix();
            proj * view * model
        };

        unsafe {
            let info = instance.info.buffer_info.get().unwrap();
            self.bind_buffers_checked(Some(info.vao), Some(info.vbo), info.ebo);
            gl::UniformMatrix4fv(
                    gl::GetUniformLocation(self.program, gl_str!("transform")), 1,
                    gl::FALSE as GLboolean, transform.as_mut_ptr());
            match instance.info.elements {
                None => { gl::DrawArrays(gl::TRIANGLES, info.start as i32, info.size as i32); },
                Some(_) => { gl::DrawElements(gl::TRIANGLES, info.size as i32,
                        gl::UNSIGNED_INT, uint_size!(info.start, CVoid)); },
            }
        }
    }
}

// Driver test program.
fn main() {
    let mut window = GameWindow::new(800, 600, "Engine Test".to_string()).unwrap();
    let camera1 = PerspectiveCamera::new(
            Vector3D::new(7.0, 7.0, 7.0), Vector3D::new(0.0, 0.0, 0.0), window.get_aspect_ratio(),
            45.0, 1.0, 100.0);
    let camera2 = PerspectiveCamera::new(
            Vector3D::new(0.00001, 0.0, 10.0), Vector3D::new(0.0, 0.0, 0.0),
            window.get_aspect_ratio(), 45.0, 0.1, 100.0);
    let main_camera = window.attach_camera(camera1);
    let secondary_camera = window.attach_camera(camera2);
    window.set_active_camera(main_camera).unwrap();
    let rb = Rc::new(ModelInfo::new_box(1.0, 1.0, 1.0, Color::new_rgb(1.0, 0.0, 0.0)));
    let ob = Rc::new(ModelInfo::new_box(1.0, 1.0, 1.0, Color::new_rgb(1.0, 0.5, 0.0)));
    let yb = Rc::new(ModelInfo::new_box(1.0, 1.0, 1.0, Color::new_rgb(1.0, 1.0, 0.0)));
    let gb = Rc::new(ModelInfo::new_box(1.0, 1.0, 1.0, Color::new_rgb(0.0, 1.0, 0.0)));
    let bb = Rc::new(ModelInfo::new_box(1.0, 1.0, 1.0, Color::new_rgb(0.0, 0.0, 1.0)));
    let mut boxes = Vec::new();
    for i in 0..5 {
        boxes.push(ModelInstance::from(rb.clone()));
        boxes.last_mut().unwrap().pos = Vector3D::new(0.0, i as f32 * 1.5, 0.0);
        boxes.push(ModelInstance::from(ob.clone()));
        boxes.last_mut().unwrap().pos = Vector3D::new(1.5, i as f32 * 1.5, 0.0);
        boxes.push(ModelInstance::from(yb.clone()));
        boxes.last_mut().unwrap().pos = Vector3D::new(3.0, i as f32 * 1.5, 0.0);
        boxes.push(ModelInstance::from(gb.clone()));
        boxes.last_mut().unwrap().pos = Vector3D::new(4.5, i as f32 * 1.5, 0.0);
        boxes.push(ModelInstance::from(bb.clone()));
        boxes.last_mut().unwrap().pos = Vector3D::new(6.0, i as f32 * 1.5, 0.0);
    }
    // let mut box_instance = ModelInstance::from(box_info.clone());

    let mut left_pressed = 0;
    let mut right_pressed = 0;
    let mut up_pressed = 0;
    let mut down_pressed = 0;
    let mut shift_pressed = 0;
    let mut last_time = time::now().to_timespec();
    let mut elapsed_time = 0.0;
    loop {
        let curr_time = time::now().to_timespec();
        let elapsed_msec = (curr_time - last_time).num_microseconds().unwrap();
        let dt = elapsed_msec as f32 / 1000000.0;
        elapsed_time += dt;
        last_time = curr_time;

        // Update Camera.
        {
            if shift_pressed == 0 {
                window.set_active_camera(main_camera).unwrap();
                // window.clear_vertex_buffers();
            } else {
                window.set_active_camera(secondary_camera).unwrap();
            }
            let x_dir = (right_pressed - left_pressed) as f32 * 5.0 * dt;
            let y_dir = (up_pressed - down_pressed) as f32 * 5.0 * dt;
            let mut camera = window.get_active_camera().unwrap();
            let cam_dir = camera.get_fwd();
            let fwd = Vector3D::new(cam_dir[0], cam_dir[1], 0.0).normalize();
            let right = camera.get_right();
            let dir = right * x_dir + fwd * -y_dir;
            camera.pos = camera.pos + dir;
            camera.target = camera.target + dir;
        }

        // Update Objects.
        let mut count = 0.0;
        for elem in &mut boxes {
            elem.scale = 0.5 + (((
                    elapsed_time * 10.0 + (((count % 5.0) / 4.0) * 3.1415)).sin() + 1.0) / 3.0);
            count += 1.0;
        }

        window.clear();
        for elem in &boxes {
            window.draw_instance(&elem);
        }
        window.swap_buffers();

        for event in window.poll_events() {
            match event {
                Event::KeyboardInput(state, _, Some(key)) => {
                    let pressed = if state == ElementState::Pressed { 1 } else { 0 };
                    match key {
                        VirtualKeyCode::Left => left_pressed = pressed,
                        VirtualKeyCode::Right => right_pressed = pressed,
                        VirtualKeyCode::Up => up_pressed = pressed,
                        VirtualKeyCode::Down => down_pressed = pressed,
                        VirtualKeyCode::LShift => shift_pressed = pressed,
                        _ => (),
                    }
                }
                Event::Closed => process::exit(0),
                _ => ()
            }
        }
        // sleep(Duration::from_millis(500));
    }
}
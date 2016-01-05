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
const VBO_SIZE: usize = 65535 - 252;

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
pub struct VBOInfo {
    pub gen: usize,
    pub start: usize,
    pub size: usize,
    pub vbo: GLuint,
}

// Stores information about the model which can be instantiated to create a ModelInstance. 
pub struct ModelInfo {
    pub vertices: Vec<GLfloat>,
    pub color: Color,
    pub mat: Material,
    pub vbo_info: Cell<Option<VBOInfo>>,
}

impl ModelInfo {
    // Default constructor with color initialized to <1.0, 1.0, 1.0, 1.0>.
    pub fn new(vertices: Vec<GLfloat>, mat: Material) -> ModelInfo {
        ModelInfo::new_with_color(vertices, Color::new_rgb(1.0, 1.0, 1.0), mat)
    }

    // Constructor to create a ModelInfo with a Color.
    pub fn new_with_color(vertices: Vec<GLfloat>, color: Color, mat: Material) -> ModelInfo {
        ModelInfo { vertices: vertices, color: color, mat: mat, vbo_info: Cell::new(None) }
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
    vao: GLuint,
    program: GLuint,
    gen: usize,
    bound_vbo: Option<GLuint>,
    vbos: Vec<(GLuint, usize)>, // (vbo_id, size)

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
                bg_color: bg_color, cameras: Vec::new(), gl_window: gl_window, vao: 0,
                program: 0, point_lights: pl, directional_lights: dl, spot_lights: sl,
                active_camera: None, gen: 0, vbos: Vec::new(), bound_vbo: None };
        // Begin unsafe OpenGL shenanigans. Here, we compile and link the shaders, set up the VAO
        // and VBO, and specify the layout of the vertex data.
        unsafe {
            let vs = shader::compile_shader("std.vert", gl::VERTEX_SHADER);
            let fs = shader::compile_shader("std.frag", gl::FRAGMENT_SHADER);
            window.program = shader::link_program(vs, fs);

            gl::GenVertexArrays(1, &mut window.vao);
            gl::BindVertexArray(window.vao);
            window.initialize_vbo();
            gl::Enable(gl::DEPTH_TEST);
            

            gl::UseProgram(window.program);
            gl::BindFragDataLocation(window.program, 0, gl_str!("out_color"));

            // let pos_attr = gl::GetAttribLocation(window.program, gl_str!("position"));
            // gl::EnableVertexAttribArray(pos_attr as GLuint);
            // gl::VertexAttribPointer(
            //         pos_attr as GLuint, 3, gl::FLOAT, gl::FALSE as GLboolean,
            //         float_size!(7, GLsizei), ptr::null());
            // let color_attr = gl::GetAttribLocation(window.program, gl_str!("color"));
            // gl::EnableVertexAttribArray(color_attr as GLuint);
            // gl::VertexAttribPointer(
            //         color_attr as GLuint, 4, gl::FLOAT, gl::FALSE as GLboolean,
            //         float_size!(7, GLsizei), float_size!(3, CVoid));
        }

        window.set_size(width, height);
        window.clear();
        window.swap_buffers();
        Ok(window)
    }

    // A helper method for binding the VBO that sets/checks the previously bound buffer.
    fn bind_vbo_checked(&mut self, vbo: GLuint) { unsafe {
        match self.bound_vbo {
            Some(b) => { if b == vbo { return }; },
            None => (),
        };
        println!("switching buffers to {}...", vbo);
        self.bound_vbo = Some(vbo);
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
    } }

    // Initializes an managed empty VBO of size VBO_SIZE and returns the handle.
    fn initialize_vbo(&mut self) { unsafe {
        let mut vbo = 0;
        gl::GenBuffers(1, &mut vbo);
        println!("generated VBO: {}", vbo);
        self.bind_vbo_checked(vbo);
        gl::BufferData(
                gl::ARRAY_BUFFER, float_size!(VBO_SIZE, GLsizeiptr),
                0 as CVoid, gl::STATIC_DRAW);

        let pos_attr = gl::GetAttribLocation(self.program, gl_str!("position"));
        gl::EnableVertexAttribArray(pos_attr as GLuint);
        gl::VertexAttribPointer(
                pos_attr as GLuint, 3, gl::FLOAT, gl::FALSE as GLboolean,
                float_size!(7, GLsizei), ptr::null());
        let color_attr = gl::GetAttribLocation(self.program, gl_str!("color"));
        gl::EnableVertexAttribArray(color_attr as GLuint);
        gl::VertexAttribPointer(
                color_attr as GLuint, 4, gl::FLOAT, gl::FALSE as GLboolean,
                float_size!(7, GLsizei), float_size!(3, CVoid));
        self.vbos.push((vbo, 0));
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

    // Private method to manage the engine's VBO space and return the VBOInfo of a particular
    // ModelInfo. If there is no associated VBOInfo for a ModelInfo, then we find an empty space in
    // the engine's VBO space and assign a new VBOInfo. If there is no more empty space in any of
    // the managed VBOs, we create a new VBO and assign it there instead. There is also a
    // generation field in both the VBOInfo and the Engine. On clear_vertex_buffers(), we increment
    // this generation count in the engine. If the generation count on the ModelInfo does not match
    // the count of the Engine, we remap.
    fn get_vbo_info(&mut self, info: Rc<ModelInfo>) {
        let vbo_info = info.vbo_info.get();
        match vbo_info {
            None => {
                // for &(vbo_num, vbo_size) in &self.vbos {
                //     if info.vertices.len() as i32 < VBO_SIZE as i32 - vbo_size as i32 {

                //     }
                //     println!("vbo_num: {}, size: {}", vbo_num, vbo_size);
                // }
            },
            Some(v) => { ()
            }
        }
    }

    // Private helper method that maps a given Rc<ModelInfo> to a VBO location.
    fn map_vbo(&mut self, info: Rc<ModelInfo>) {
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

        let mut index = -1;
        for (i, vbo_pair) in self.vbos.iter().enumerate() {
            if (vertices.len() as i32) < (VBO_SIZE as i32) - (vbo_pair.1 as i32) {
                index = i as i32;
                break;
            }
        }
        if index == -1 {
            self.initialize_vbo();
            index = (self.vbos.len() - 1) as i32;
        }
        let vbo_pair = self.vbos[index as usize];
        let new_vbo = VBOInfo {
                start: vbo_pair.1, size: vertices.len(), gen: self.gen, vbo: vbo_pair.0 };
        self.bind_vbo_checked(new_vbo.vbo);
        unsafe {
            gl::BufferSubData(
                    gl::ARRAY_BUFFER, float_size!(new_vbo.start, GLintptr),
                    float_size!(new_vbo.size, GLsizeiptr), vec_to_addr!(vertices));
        }
        info.vbo_info.set(Some(new_vbo));
        self.vbos[index as usize] = (vbo_pair.0, vbo_pair.1 + vertices.len());
    }

    // Draw a ModelInstance to the window using a camera, position, vertices, and materials.
    // TODO: We are currently doing some immediate mode-esque rendering by discarding the VBO every
    // draw. This is pretty bad and is going to be a bottleneck. Look into having the GameWindow
    // manage the VBO data.
    // The current plan is to:
    // 1. add VBO, start, size, and generation fields to ModelInfo
    // 2. allocate 65535 sized VBOs on draw_instance. If (start, size) is None, then allocate.
    // 3. if there is no more room, make new VBO (and if size is > 65535, allocate for that size).
    // 4. add a generation int and increment it on clear_vertex_buffers(). generation must be the
    //    same when draw_instance() or else must remap.
    pub fn draw_instance(&mut self, instance: &ModelInstance) {
        // let vbo_info = match instance.info.vbo_info.get() {
        //     None => {
        //         let num_vertices = instance.info.vertices.len();
        //         let mut new_vbo_info = VBOInfo { start: 0, size: 0, gen: self.gen };
        //         instance.info.vbo_info.set(Some(new_vbo_info));
        //         new_vbo_info
        //     },
        //     Some(v) => v
        // };
        // vbo_info.start = 0;
        self.map_vbo(instance.info.clone());
        // println!("COLOR: ({}, {}, {})  |  VBO: {}  |  VBO_START: {}", instance.info.color.r, instance.info.color.g, instance.info.color.b, instance.info.vbo_info.get().unwrap().vbo, instance.info.vbo_info.get().unwrap().start);

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
            // let mut vertices: Vec<GLfloat> = Vec::new();
            // for x in 0..instance.info.vertices.len() {
            //     if x % 3 != 0 {
            //         continue;
            //     }
            //     vertices.push(instance.info.vertices[x]);
            //     vertices.push(instance.info.vertices[x + 1]);
            //     vertices.push(instance.info.vertices[x + 2]);
            //     vertices.push(instance.info.color.r);
            //     vertices.push(instance.info.color.g);
            //     vertices.push(instance.info.color.b);
            //     vertices.push(instance.info.color.a);
            // }
            // gl::BufferData(
            //     gl::ARRAY_BUFFER, float_size!(vertices.len(), GLsizeiptr),
            //     vec_to_addr!(vertices), gl::STATIC_DRAW);

            let vbo_info = instance.info.vbo_info.get().unwrap();
            self.bind_vbo_checked(vbo_info.vbo);
            gl::UniformMatrix4fv(
                    gl::GetUniformLocation(self.program, gl_str!("transform")), 1,
                    gl::FALSE as GLboolean, transform.as_mut_ptr());
            gl::DrawArrays(gl::TRIANGLES, (vbo_info.start / 7) as i32, (vbo_info.size / 7) as i32);
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
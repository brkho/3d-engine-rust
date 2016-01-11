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

use cgmath::{Point, Matrix, EuclideanVector, SquareMatrix};
use gl::types::*;
use glutin::{Window, Event, VirtualKeyCode, ElementState};
use mmo::util::{bmp, obj, shader};
use std::cell::Cell;
use std::cmp;
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

// Number of elements in a VBO or EBO.
const BUFFER_SIZE: usize = 65535 * 4;

// Maximum number of dynamic lights in a scene.
const MAX_LIGHTS: usize = 8;

// Contents of a VBO.
// [P_x  P_y  P_z  N_x  N_y  N_z  T_u  T_v]
const VERTEX_POS_SIZE: usize = 3;
const VERTEX_NORMAL_SIZE: usize = 3;
const VERTEX_TCOORD_SIZE: usize = 2;
const VERTEX_SIZE: usize = VERTEX_POS_SIZE + VERTEX_NORMAL_SIZE + VERTEX_TCOORD_SIZE;

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

// Describes a material for a model that contains a color, diffuse map, specular map, and a
// shininess factor for specular. This can only be created after the window context is set up.
pub struct Material {
    pub color: Color,
    pub diffuse: GLuint,
    pub specular: GLuint,
    pub shininess: GLfloat,
}

impl Material {
    // Default constructor that automatically assigns a white color given a shininess and paths to
    // the diffuse and specular maps as BMPs.
    pub fn new(diffuse_name: Option<String>, specular_name: Option<String>, shininess: GLfloat)
            -> Material {
        Material::new_with_color(diffuse_name, specular_name,
                Color::new_rgb(1.0, 1.0, 1.0), shininess)
    }

    // Reads and binds a BMP texture given a name and returns the corresponding texture ID.
    fn read_and_bind(texture_name: Option<String>) -> GLuint { unsafe {
        if let Some(name) = texture_name {
            let texture = bmp::decode_bmp(name).unwrap();
            let image = texture.get_rgba_vec();
            let mut texture_id = 0;
            gl::GenTextures(1, &mut texture_id);
            gl::BindTexture(gl::TEXTURE_2D, texture_id);
            gl::TexImage2D(
                    gl::TEXTURE_2D, 0, gl::RGBA as GLsizei, texture.width as GLsizei,
                    texture.height as GLint, 0, gl::RGBA as GLuint, gl::UNSIGNED_BYTE,
                    vec_to_addr!(image));
            gl::GenerateMipmap(gl::TEXTURE_2D);
            texture_id
        } else { 0 }
    }}

    // Creates a Material with paths to diffuse and specular maps, shiniess, and color.
    pub fn new_with_color(diffuse_name: Option<String>, specular_name: Option<String>,
            color: Color, shininess: GLfloat) -> Material {
        let diffuse = Material::read_and_bind(diffuse_name);
        let specular = Material::read_and_bind(specular_name);
        Material {color: color, diffuse: diffuse, specular: specular, shininess: shininess }
    }
}

#[derive(Copy, Clone)]
pub struct BufferInfo {
    pub gen: usize,
    pub start: usize,
    pub size: usize,
    pub vao: GLuint,
}

// Stores information about the model which can be instantiated to create a ModelInstance. 
pub struct ModelInfo {
    pub vertices: Vec<GLfloat>,
    pub normals: Vec<GLfloat>,
    pub elements: Vec<GLuint>,
    pub tcoords: Vec<GLfloat>,
    pub mat: Material,
    pub buffer_info: Cell<Option<BufferInfo>>,
}

impl ModelInfo {
    // Default constructor with a material.
    pub fn new(vertices: Vec<GLfloat>, elems: Vec<GLuint>, normals: Vec<GLfloat>,
            tcoords: Vec<GLfloat>, mat: Material) -> ModelInfo {
        ModelInfo { vertices: vertices, normals: normals, elements: elems, tcoords: tcoords,
                mat: mat, buffer_info: Cell::new(None) }
    }

    // Creates a box with specified size and color.
    pub fn new_box(scale_x: f32, scale_y: f32, scale_z: f32, mat: Material) -> ModelInfo {
        let vertices: Vec<GLfloat> = vec![
                -0.5 * scale_x, -0.5 * scale_y, -0.5 * scale_z,
                 0.5 * scale_x, -0.5 * scale_y, -0.5 * scale_z,
                 0.5 * scale_x,  0.5 * scale_y, -0.5 * scale_z,
                -0.5 * scale_x,  0.5 * scale_y, -0.5 * scale_z,
                -0.5 * scale_x, -0.5 * scale_y,  0.5 * scale_z,
                 0.5 * scale_x, -0.5 * scale_y,  0.5 * scale_z,
                 0.5 * scale_x,  0.5 * scale_y,  0.5 * scale_z,
                -0.5 * scale_x,  0.5 * scale_y,  0.5 * scale_z,
                -0.5 * scale_x,  0.5 * scale_y, -0.5 * scale_z,
        ];
        let elements: Vec<GLuint> = vec![
                0, 1, 2, 2, 3, 0, 4, 5, 6, 6, 7, 4, 7, 3, 0, 0, 4, 7, 6, 2, 1, 1, 5, 6, 0,
                1, 5, 5, 4, 0, 3, 2, 6, 6, 7, 8,
        ];
        let normals: Vec<GLfloat> = vec![0.0; 9 * 3];
        let uvs: Vec<GLfloat> = vec![0.0; 9 * 2];
        ModelInfo::new(vertices, elements, normals, uvs, mat)
    }

    // Creates an ModelInfo from the result of a OBJ decoding.
    pub fn from_obj(object: &obj::DecodedOBJ, mat: Material) -> ModelInfo {
        let mut vertices: Vec<GLfloat> = Vec::new();
        let mut normals: Vec<GLfloat> = Vec::new();
        let mut tcoords: Vec<GLfloat> = Vec::new();
        let mut elements: Vec<GLuint> = Vec::new();
        for vertex in &object.vertices {
            vertices.push(vertex.pos.x);
            vertices.push(vertex.pos.y);
            vertices.push(vertex.pos.z);
            normals.push(vertex.norm.x);
            normals.push(vertex.norm.y);
            normals.push(vertex.norm.z);
            tcoords.push(vertex.tc.x);
            tcoords.push(vertex.tc.y);
        }
        for element in &object.elements {
            elements.push(element.0);
            elements.push(element.1);
            elements.push(element.2);
        }
        ModelInfo::new(vertices, elements, normals, tcoords, mat)
    }

    // Gets a single vector representing the the ModelInfo in VBO format.
    pub fn get_vbo_format(&self) -> Vec<GLfloat> {
        let mut vertices: Vec<GLfloat> = Vec::new();
        let mut y = 0;
        for x in 0..self.vertices.len() {
            if x % 3 != 0 {
                continue;
            }
            vertices.push(self.vertices[x]);
            vertices.push(self.vertices[x + 1]);
            vertices.push(self.vertices[x + 2]);
            vertices.push(self.normals[x]);
            vertices.push(self.normals[x + 1]);
            vertices.push(self.normals[x + 2]);
            vertices.push(self.tcoords[y]);
            vertices.push(self.tcoords[y + 1]);
            y += 2;
        }
        vertices
    }
}

// An instantiazation of a ModelInfo that represents a model in-game. This has a variety of
// positional attributes used to render the instance.
pub struct ModelInstance {
    pub info: Rc<ModelInfo>,
    pub pos: Vector3D,
    pub rot: Quaternion,
    pub scale: f32,
    pub model: cgmath::Matrix4<GLfloat>,
    pub normal: cgmath::Matrix4<GLfloat>,
}

impl ModelInstance {
    // Create an instance from a reference counted pointer to a ModelInfo struct.
    pub fn from(info: Rc<ModelInfo>) -> ModelInstance {
        let pos = Vector3D::new(0.0, 0.0, 0.0);
        let rot = Quaternion::new(1.0, 0.0, 0.0, 0.0);
        let scale = 1.0;
        let model = cgmath::Matrix4::from(cgmath::Decomposed {
                scale: scale, rot: rot, disp: pos });
        let norm = model.clone().invert().unwrap().transpose();
        ModelInstance { info: info, pos: pos, scale: scale, rot: rot, model: model, normal: norm }
    }

    // Updates the model and normal matrices. This must be called after any sequence of struct
    // field changes for the changes to appear in-world.
    pub fn update(&mut self) {
        let model = cgmath::Matrix4::from(cgmath::Decomposed {
                scale: self.scale, rot: self.rot, disp: self.pos });
        let normal = model.clone().invert().unwrap().transpose();
        self.model = model;
        self.normal = normal;
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
    view: cgmath::Matrix4<GLfloat>,
    up: Vector3D,
}

// Implementation of the Camera methods for PerspectiveCamera.
impl Camera for PerspectiveCamera {
    // Return the precomputed view matrix.
    fn get_view_matrix(&self) -> cgmath::Matrix4<GLfloat> { self.view }

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

    // Return the precomputed projection matrix.
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
    // TODO: Right now, you must call update directly after. I should factor this new code into the
    // GameWindow class like I do with lights.
    pub fn new_with_up(pos: Vector3D, target: Vector3D, up: Vector3D, aspect: f32,
            fov: f32, near: f32, far: f32) -> PerspectiveCamera {
        let proj = cgmath::PerspectiveFov {
                fovy: cgmath::Rad::from(cgmath::deg(fov)),
                aspect: aspect,
                near: near,
                far: far };
        let dummy_view = cgmath::Matrix4::identity();
        PerspectiveCamera { pos: pos, target: target,
                up: up, proj: cgmath::Matrix4::from(proj), view: dummy_view }
    }

    // Updates the camera view matrix and the view uniform on the GPU. This must be called after
    // any sequence of struct field changes for the changes to appear in-world.
    pub fn update(&mut self, program: u32) {
        self.view = cgmath::Matrix4::look_at(
                cgmath::Point3::from_vec(self.pos),
                cgmath::Point3::from_vec(self.target),
                self.up);
        unsafe { uniform_vec3!(program, "camera", v3d_to_vec!(self.pos)) };
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
    pub light_index: usize,
}

impl PointLight {
    // Updates the light uniforms on the GPU. This must be called after any sequence of struct
    // field changes for the changes to appear in-world.
    pub fn update(&self, program: u32) { unsafe {
        let li = self.light_index;
        uniform_uint!(program, lights![li, "type"], 1);
        let color = vec![self.intensity.r, self.intensity.g, self.intensity.b];
        uniform_vec3!(program, lights![li, "intensity"], color);
        uniform_vec3!(program, lights![li, "position"], v3d_to_vec!(self.position));
        uniform_float!(program, lights![li, "const_attn"], self.const_attn);
        uniform_float!(program, lights![li, "linear_attn"], self.linear_attn);
        uniform_float!(program, lights![li, "quad_attn"], self.quad_attn);
    }}
}

// Light source that shines from an infinite distance from a direction (such as the sun).
pub struct DirectionalLight {
    pub intensity: Color,
    pub direction: Vector3D,
    pub light_index: usize,
}

impl DirectionalLight {
    // Updates the light uniforms on the GPU. This must be called after any sequence of struct
    // field changes for the changes to appear in-world.
    pub fn update(&self, program: u32) { unsafe {
        let li = self.light_index;
        uniform_uint!(program, lights![li, "type"], 2);
        let color = vec![self.intensity.r, self.intensity.g, self.intensity.b];
        println!("INTENSITY: {:?}, light_index: {}", color, self.light_index);
        uniform_vec3!(program, lights![li, "intensity"], color);
        uniform_vec3!(program, lights![li, "direction"], v3d_to_vec!(self.direction));
    }}
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
    pub light_index: usize,
}

impl SpotLight {
    // Updates the light uniforms on the GPU. This must be called after any sequence of struct
    // field changes for the changes to appear in-world.
    pub fn update(&self, program: u32) { unsafe {
        let li = self.light_index;
        uniform_uint!(program, lights![li, "type"], 3);
        let color = vec![self.intensity.r, self.intensity.g, self.intensity.b];
        uniform_vec3!(program, lights![li, "intensity"], color);
        uniform_vec3!(program, lights![li, "position"], v3d_to_vec!(self.position));
        uniform_vec3!(program, lights![li, "direction"], v3d_to_vec!(self.direction));
        uniform_float!(program, lights![li, "const_attn"], self.const_attn);
        uniform_float!(program, lights![li, "linear_attn"], self.linear_attn);
        uniform_float!(program, lights![li, "quad_attn"], self.quad_attn);
        uniform_float!(program, lights![li, "cutoff"], self.cutoff);
        uniform_float!(program, lights![li, "dropoff"], self.dropoff);
    }}
}

// A window for graphics drawing that is managed by the graphics module. This is a thin wrapper
// around the glutin Window class and will manage draws to the glutin window.
pub struct GameWindow {
    pub bg_color: Color,
    pub cameras: Vec<Option<PerspectiveCamera>>,
    active_camera: Option<usize>,
    gl_window: Window,
    point_lights: Vec<Option<PointLight>>, // (light_index, light)
    directional_lights: Vec<Option<DirectionalLight>>, // (light_index, light)
    spot_lights: Vec<Option<SpotLight>>, // (light_index, light)
    light_indices: Vec<usize>,
    program: GLuint,
    gen: usize,
    working_vao: GLuint,
    bound_vao: Option<GLuint>,
    default_texture: GLuint,
    vaos: Vec<Vec<Option<GLuint>>>,
    vbos: Vec<(GLuint, usize, usize)>, // (vbo_id, size, max_size)
    ebos: Vec<(GLuint, usize, usize)>, // (ebo_id, size, max_size)

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
        let lights:Vec<usize> = (0..MAX_LIGHTS).collect();

        let mut window = GameWindow {
                bg_color: bg_color, cameras: Vec::new(), gl_window: gl_window,
                program: 0, point_lights: pl, directional_lights: dl, spot_lights: sl,
                active_camera: None, gen: 0, bound_vao: None, vbos: Vec::new(), ebos: Vec::new(),
                vaos: Vec::new(), working_vao: 0, light_indices: lights, default_texture: 0};
        // Begin unsafe OpenGL shenanigans. Here, we compile and link the shaders, set up the VAO
        // and VBO, and set some texture parameters.
        unsafe {
            let vs = shader::compile_shader("std.vert", gl::VERTEX_SHADER);
            let fs = shader::compile_shader("std.frag", gl::FRAGMENT_SHADER);
            window.program = shader::link_program(vs, fs);
            gl::GenVertexArrays(1, &mut window.working_vao);
            window.initialize_vbo(0);
            window.initialize_ebo(0);
            gl::Enable(gl::DEPTH_TEST);
            gl::UseProgram(window.program);
            gl::BindFragDataLocation(window.program, 0, gl_str!("out_color"));

            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_BORDER as GLint);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_BORDER as GLint);
            gl::TexParameteri(
                    gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR_MIPMAP_NEAREST as GLint);
            gl::TexParameteri(
                    gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR_MIPMAP_NEAREST as GLint);
            // Set up the default white texture.
            let white_tex: Vec<u8> = vec![255, 255, 255];
            gl::GenTextures(1, &mut window.default_texture);
            gl::BindTexture(gl::TEXTURE_2D, window.default_texture);
            gl::TexImage2D(
                gl::TEXTURE_2D, 0, gl::RGB as GLsizei, 1, 1, 0, gl::RGB as GLuint,
                gl::UNSIGNED_BYTE, vec_to_addr!(white_tex));
            gl::GenerateMipmap(gl::TEXTURE_2D);
            gl::BindTexture(gl::TEXTURE_2D, 0);
        }

        window.set_size(width, height);
        window.clear();
        window.swap_buffers();
        Ok(window)
    }

    // A helper method for binding the VAO and VBO that sets/checks the previously bound buffer.
    fn bind_vao_checked(&mut self, vao: GLuint) { unsafe {
        if match self.bound_vao {
                Some(bv) => bv != vao,
                None => true, } {
            self.bound_vao = Some(vao);
            gl::BindVertexArray(vao);
        }
    } }

    // Initializes a managed empty VBO of size BUFFER_SIZE and adds it to the vector of VBOs. This
    // also adds an uninitialized row to the VAOs data structure. This takes a max argument to
    // still not fail on creation even if we create a VBO for greater than BUFFER_SIZE elems.
    fn initialize_vbo(&mut self, max: usize) { unsafe {
        let buffer_size = cmp::max(max, BUFFER_SIZE);
        let working_vao = self.working_vao.clone();
        self.bind_vao_checked(working_vao);
        let mut vbo = 0;
        gl::GenBuffers(1, &mut vbo);
        println!("initializing vbo {}", vbo);
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::BufferData(
                gl::ARRAY_BUFFER, float_size!(buffer_size, GLsizeiptr),
                0 as CVoid, gl::STATIC_DRAW);
        self.vbos.push((vbo, 0, buffer_size));
        let size = if self.vaos.is_empty() { 0 } else { self.vaos[0].len() };
        self.vaos.push(vec![None; size]);
    }}

    // Initializes a managed empty EBO of size BUFFER_SIZE and adds it to the vector of EBOs. This
    // also adds an uninitialized column to the VAOs data structure. This takes a max argument to
    // still not fail on creation even if we create a EBO for greater than BUFFER_SIZE elems.
    fn initialize_ebo(&mut self, max: usize) { unsafe {
        let buffer_size = cmp::max(max, BUFFER_SIZE);
        let working_vao = self.working_vao.clone();
        self.bind_vao_checked(working_vao);
        let mut ebo = 0;
        gl::GenBuffers(1, &mut ebo);
        println!("initializing ebo: {}", ebo);
        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
        gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER, uint_size!(buffer_size, GLsizeiptr),
                0 as CVoid, gl::STATIC_DRAW);
        self.ebos.push((ebo, 0, buffer_size));
        for vao_vec in self.vaos.iter_mut() {
            vao_vec.push(None);
        }
    }}

    // Initializes a VAO if there is not already an existing one for the EBO/VBO combination and
    // returns the corresponding VAO ID.
    fn initialize_vao(&mut self, vbo: usize, ebo: usize) -> GLuint { unsafe {
        match self.vaos[vbo as usize][ebo as usize] {
            Some(id) => id,
            None => {
                let mut vao = 0;
                gl::GenVertexArrays(1, &mut vao);
                self.bind_vao_checked(vao);
                self.vaos[vbo][ebo] = Some(vao);
                gl::BindBuffer(gl::ARRAY_BUFFER, self.vbos[vbo].0);
                gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.ebos[ebo].0);
                let pos_attr = gl::GetAttribLocation(self.program, gl_str!("position"));
                gl::EnableVertexAttribArray(pos_attr as GLuint);
                gl::VertexAttribPointer(
                        pos_attr as GLuint, VERTEX_POS_SIZE as i32, gl::FLOAT,
                        gl::FALSE as GLboolean, float_size!(VERTEX_SIZE, GLsizei), ptr::null());
                let normal_attr = gl::GetAttribLocation(self.program, gl_str!("normal"));
                gl::EnableVertexAttribArray(normal_attr as GLuint);
                gl::VertexAttribPointer(
                        normal_attr as GLuint, VERTEX_NORMAL_SIZE as i32, gl::FLOAT,
                        gl::FALSE as GLboolean, float_size!(VERTEX_SIZE, GLsizei),
                        float_size!(VERTEX_POS_SIZE, CVoid));
                let tcoord_attr = gl::GetAttribLocation(self.program, gl_str!("tcoord"));
                gl::EnableVertexAttribArray(tcoord_attr as GLuint);
                gl::VertexAttribPointer(
                        tcoord_attr as GLuint, VERTEX_TCOORD_SIZE as i32, gl::FLOAT,
                        gl::FALSE as GLboolean, float_size!(VERTEX_SIZE, GLsizei),
                        float_size!(VERTEX_POS_SIZE + VERTEX_NORMAL_SIZE, CVoid));
                let color_attr = gl::GetAttribLocation(self.program, gl_str!("color"));
                gl::EnableVertexAttribArray(color_attr as GLuint);
                vao
            },
        }
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
            linear_attn: f32, quad_attn: f32) -> Option<usize> {
        let index = match self.light_indices.pop() { None => { return None; }, Some(i) => i, };
        let light = PointLight {
                intensity: intensity, position: position, const_attn: const_attn,
                linear_attn: linear_attn, quad_attn: quad_attn, light_index: index };
        light.update(self.program);
        Some(GameWindow::add_light(light, &mut self.point_lights))
    }

    // Removes a PointLight from the scene given its handle.
    pub fn remove_point_light(&mut self, index: usize) {
        let free_index = (&self.point_lights[index]).as_ref().unwrap().light_index;
        unsafe { uniform_uint!(self.program, lights![free_index, "type"], 0); };
        self.light_indices.push(free_index);
        self.point_lights[index] = None;
    }

    // Gets a reference to a PointLight given its handle.
    pub fn get_point_light(&mut self, index: usize) -> &mut PointLight {
        (&mut self.point_lights[index]).as_mut().unwrap()
    }

    // Constructs and adds a DirectionalLight to the scene. This then returns an u16 handle
    // (internally representing the index in the array) that can be used with the getter to modify
    // light attrs.
    pub fn add_directional_light(&mut self, intensity: Color,
            direction: Vector3D) -> Option<usize> {
        let index = match self.light_indices.pop() { None => { return None; }, Some(i) => i, };
        let light = DirectionalLight { intensity: intensity, direction: direction,
                light_index: index };
        light.update(self.program);
        Some(GameWindow::add_light(light, &mut self.directional_lights))
    }

    // Removes a DirectionalLight from the scene given its handle.
    pub fn remove_directional_light(&mut self, index: usize) {
        let free_index = (&self.directional_lights[index]).as_ref().unwrap().light_index;
        unsafe { uniform_uint!(self.program, lights![free_index, "type"], 0); };
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
            dropoff: f32) -> Option<usize> {
        let index = match self.light_indices.pop() { None => { return None; }, Some(i) => i, };
        let light = SpotLight {
                intensity: intensity, position: position, const_attn: const_attn,
                direction: direction, linear_attn: linear_attn, quad_attn: quad_attn,
                cutoff: cutoff, dropoff: dropoff, light_index: index };
        light.update(self.program);
        Some(GameWindow::add_light(light, &mut self.spot_lights))
    }

    // Removes a SpotLight from the scene given its handle.
    pub fn remove_spot_light(&mut self, index: usize) {
        let free_index = (&self.spot_lights[index]).as_ref().unwrap().light_index;
        unsafe { uniform_uint!(self.program, lights![free_index, "type"], 0); };
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
        let vertices = info.get_vbo_format();
        // Find empty EBO space.
        let ebo_index = {
            let mut index = None;
            for (i, ebo_pair) in self.ebos.iter().enumerate() {
                if info.elements.len() < ebo_pair.2 - ebo_pair.1 {
                    index = Some(i);
                    break;
                }
            }
            match index {
                None => {
                    self.initialize_ebo(info.elements.len() + 1);
                    self.ebos.len() - 1
                },
                Some(i) => i,
            }
        };

        // Find empty VBO space.
        let vbo_index = {
            let mut index = None;
            for (i, vbo_pair) in self.vbos.iter().enumerate() {
                if vertices.len() < vbo_pair.2 - vbo_pair.1 {
                    index = Some(i);
                    break;
                }
            }
            match index {
                None => {
                    self.initialize_vbo(vertices.len() + 1);
                    self.vbos.len() - 1
                },
                Some(i) => i,
            }
        };

        let vao = self.initialize_vao(vbo_index, ebo_index);
        let vbo_pair = self.vbos[vbo_index];
        let ebo_pair = self.ebos[ebo_index];
                println!("Mapped LEN: {} to VBO: {}, EBO: {}, VAO: {}", vertices.len(), vbo_pair.0, ebo_pair.0, vao);

        let mut elements: Vec<GLuint> = Vec::new();
        for elem in &info.elements {
            elements.push(elem.clone() + (vbo_pair.1 as GLuint / VERTEX_SIZE as GLuint));
        }
        let buffer_info = BufferInfo {
                start: ebo_pair.1, size: elements.len(), gen: self.gen, vao: vao };
        info.buffer_info.set(Some(buffer_info));
        unsafe {
            let working_vao = self.working_vao.clone();
            self.bind_vao_checked(working_vao);
            gl::BufferSubData(
                    gl::ELEMENT_ARRAY_BUFFER, uint_size!(ebo_pair.1, GLintptr),
                    uint_size!(elements.len(), GLsizeiptr), vec_to_addr!(elements));
            gl::BufferSubData(
                    gl::ARRAY_BUFFER, float_size!(vbo_pair.1, GLintptr),
                    float_size!(vertices.len(), GLsizeiptr), vec_to_addr!(vertices));
        }
        self.ebos[ebo_index] = (ebo_pair.0, ebo_pair.1 + elements.len(), ebo_pair.2);
        self.vbos[vbo_index] = (vbo_pair.0, vbo_pair.1 + vertices.len(), vbo_pair.2);
    }

    // Clears the VBO/VAO/EBOs so that every ModelInfo currently mapped to the engine's VBO space
    // rmust be emapped on the next draw_instance() call.
    pub fn clear_vertex_buffers(&mut self) {
        let working_vao = self.working_vao.clone();
        self.bind_vao_checked(working_vao);
        self.gen += 1;
        for vbo_pair in self.vbos.iter_mut() {
            *vbo_pair = (vbo_pair.0, 0, vbo_pair.2);
        }
        for ebo_pair in self.ebos.iter_mut() {
            *ebo_pair = (ebo_pair.0, 0, ebo_pair.2);
        }
        for row in self.vaos.iter_mut() {
            for column in row.iter_mut() {
                if let Some(id) = *column {
                    let mut vao = id.clone();
                    unsafe { gl::DeleteVertexArrays(1, &mut vao); };
                    *column = None;
                }
            }
        }
    }

    // Draw a ModelInstance to the window using a camera, position, vertices, and materials.
    // This method also manages the engine's VBO space and updates the BufferInfo of the instance's
    // ModelInfo. If there is no associated BufferInfo for a ModelInfo, then we find an empty space
    // in the engine's VBO space and assign a new BufferInfo. If there is no more empty space in
    // any of the managed VBOs, we create a new VBO and assign it there instead. There is also a
    // generation field in both the BufferInfo and the Engine. On clear_vertex_buffers(), we
    // increment this generation count in the engine. If the generation count on the ModelInfo does
    // not match the count of the Engine, we remap.
    pub fn draw_instance(&mut self, instance: &ModelInstance) {
        match instance.info.buffer_info.get() {
            None => { self.map_vbo(instance.info.clone()); },
            Some(i) => { if i.gen != self.gen { self.map_vbo(instance.info.clone()) }; },
        }

        let transform = {
            let camera = match self.active_camera {
                None => { return; },
                Some(c) => self.cameras[c].as_ref().unwrap(),
            };
            let view = camera.get_view_matrix();
            let proj = camera.get_projection_matrix();
            proj * view * instance.model
        };

        unsafe {
            let mat = &instance.info.mat;
            let info = instance.info.buffer_info.get().unwrap();
            self.bind_vao_checked(info.vao);
            uniform_mat4!(self.program, "transform", transform);
            uniform_mat4!(self.program, "model", instance.model);
            uniform_mat4!(self.program, "normal", instance.normal);
            gl::ActiveTexture(gl::TEXTURE0);
            let diffuse_id = if mat.diffuse == 0 { self.default_texture } else { mat.diffuse };
            gl::BindTexture(gl::TEXTURE_2D, diffuse_id);
            uniform_int!(self.program, "diffuse_map", 0);
            gl::ActiveTexture(gl::TEXTURE1);
            let spec_id = if mat.specular == 0 { self.default_texture } else { mat.specular };
            gl::BindTexture(gl::TEXTURE_2D, spec_id);
            uniform_int!(self.program, "specular_map", 1);
            uniform_vec4!(self.program, "color", color_to_vec!(mat.color));

            gl::DrawElements(gl::TRIANGLES, info.size as i32,
                    gl::UNSIGNED_INT, uint_size!(info.start, CVoid));
        }
    }
}

// Driver test program.
fn main() {
    let ground = obj::decode_obj("ground.obj").unwrap();
    let bunny = obj::decode_obj("bunny_uv.obj").unwrap();
    let budda = obj::decode_obj("budda.obj").unwrap();
    let dragon = obj::decode_obj("dragon.obj").unwrap();
    let mut window = GameWindow::new(800, 600, "Engine Test".to_string()).unwrap();
    let program = window.program;
    window.bg_color = Color::new_rgb(0.2, 0.2, 0.2);

    let mut camera1 = PerspectiveCamera::new(
            Vector3D::new(17.0, 17.0, 17.0), Vector3D::new(0.0, 0.0, 0.0), window.get_aspect_ratio(),
            45.0, 1.0, 100.0);
    camera1.update(program);
    let mut camera2 = PerspectiveCamera::new(
            Vector3D::new(0.00001, 0.0, 30.0), Vector3D::new(0.0, 0.0, 0.0),
            window.get_aspect_ratio(), 45.0, 0.1, 100.0);
    camera2.update(program);
    let main_camera = window.attach_camera(camera1);
    let secondary_camera = window.attach_camera(camera2);
    window.set_active_camera(main_camera).unwrap();

    let bunny_mat = Material::new_with_color(Some("brian.bmp".to_string()), None, Color::new_rgb(1.0, 1.0, 1.0), 75.0);
    let bunny_info = Rc::new(ModelInfo::from_obj(&bunny, bunny_mat));
    let mut bunny_inst = ModelInstance::from(bunny_info.clone());
    bunny_inst.scale = 35.0;
    bunny_inst.pos = Vector3D::new(-4.0, 4.0, 0.0);
    bunny_inst.update();

    let ground_mat = Material::new_with_color(Some("brian.bmp".to_string()), None, Color::new_rgb(1.0, 1.0, 1.0), 75.0);
    // let ground_mat = Material::new_with_color(None, None, Color::new_rgb(1.0, 1.0, 1.0), 75.0);
    let ground_info = Rc::new(ModelInfo::from_obj(&ground, ground_mat));
    let mut ground_inst = ModelInstance::from(ground_info.clone());
    ground_inst.scale = 3.0;
    ground_inst.pos = Vector3D::new(0.0, 0.0, -1.5);
    ground_inst.update();

    let dragon_mat = Material::new_with_color(Some("uvs.bmp".to_string()), None, Color::new_rgb(1.0, 1.0, 1.0), 75.0);
    let dragon_info = Rc::new(ModelInfo::from_obj(&dragon, dragon_mat));
    let mut dragon_inst = ModelInstance::from(dragon_info.clone());
    dragon_inst.scale = 0.6;
    dragon_inst.pos = Vector3D::new(4.0, -4.0, 0.0);
    dragon_inst.update();

    let budda_mat = Material::new_with_color(Some("brian.bmp".to_string()), None, Color::new_rgb(1.0, 1.0, 1.0), 75.0);
    let budda_info = Rc::new(ModelInfo::from_obj(&budda, budda_mat));
    let mut budda_inst = ModelInstance::from(budda_info.clone());
    budda_inst.pos = Vector3D::new(3.5, 3.5, 1.0);
    budda_inst.update();

    let lb_mat = Material::new_with_color(None, None, Color::new_rgb(0.0, 0.0, 0.0), 75.0);
    let lb = Rc::new(ModelInfo::new_box(1.0, 1.0, 1.0, lb_mat));
    let mut lb1_inst = ModelInstance::from(lb.clone());
    lb1_inst.update();

    let mut lb2_inst = ModelInstance::from(lb.clone());
    lb2_inst.update();

    // let spot_light = window.add_spot_light(
    //         Color::new_rgb(0.3, 0.3, 0.3), Vector3D::new(0.0, 15.0, 15.0),
    //         Vector3D::new(0.0, -1.0, -1.0), 1.0, 0.0, 0.0, 0.4, 42.0).unwrap();

    // let dir_light = window.add_directional_light(
    //         Color::new_rgb(0.4, 0.4, 0.4), Vector3D::new(-1.0, -1.0, -1.0)).unwrap();

    let point_light1 = window.add_point_light(
            Color::new_rgb(1.0, 1.0, 1.0), Vector3D::new(3.0, 3.0, 1.0), 1.0, 0.06, 0.008).unwrap();
    println!("created light: {}", point_light1);

    let point_light2 = window.add_point_light(
            Color::new_rgb(1.0, 1.0, 1.0), Vector3D::new(3.0, 3.0, 1.0), 1.0, 0.06, 0.008).unwrap();
    println!("created light: {}", point_light2);

    let mut left_pressed = 0;
    let mut right_pressed = 0;
    let mut up_pressed = 0;
    let mut down_pressed = 0;
    let mut shift_pressed = 0;
    let mut last_time = time::now().to_timespec();
    let mut elapsed_time = 0.0;
    let mut frame_count = 0;
    loop {
        frame_count += 1;
        let curr_time = time::now().to_timespec();
        let elapsed_msec = (curr_time - last_time).num_microseconds().unwrap();
        let dt = elapsed_msec as f32 / 1000000.0;
        elapsed_time += dt;
        last_time = curr_time;
        if ((elapsed_time - dt) % 3.0) > (elapsed_time % 3.0) {
            println!("AVERAGE FPS: {}", frame_count as f32 / elapsed_time);
        }

        // Update Camera.
        {
            if shift_pressed == 0 {
                window.set_active_camera(main_camera).unwrap();
            } else {
                window.set_active_camera(secondary_camera).unwrap();
                // window.clear_vertex_buffers();
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
            camera.update(program);
        }

        // Update Objects.
        lb1_inst.pos = Vector3D::new(10.0 * elapsed_time.cos(), 10.0 * elapsed_time.sin(), 4.0);
        lb1_inst.update();
        lb2_inst.pos = Vector3D::new(0.0, 10.0 * (1.43 * elapsed_time).sin(), 10.0 * (1.43 * elapsed_time).cos());
        lb2_inst.update();

        {
            let mut light = window.get_point_light(point_light1);
            let lpos = Vector3D::new(10.0 * elapsed_time.cos(), 10.0 * elapsed_time.sin(), 4.0);
            light.position = lpos;
            light.update(program);
        }

        {
            let mut light = window.get_point_light(point_light2);
            let lpos = Vector3D::new(0.0, 10.0 * (1.43 * elapsed_time).sin(), 10.0 * (1.43 * elapsed_time).cos());
            light.position = lpos;
            light.update(program);
        }

        // Draw Objects.
        window.clear();
        window.draw_instance(&bunny_inst);
        window.draw_instance(&lb1_inst);
        window.draw_instance(&lb2_inst);
        window.draw_instance(&ground_inst);
        window.draw_instance(&budda_inst);
        window.draw_instance(&dragon_inst);
        // window.swap_buffers();

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
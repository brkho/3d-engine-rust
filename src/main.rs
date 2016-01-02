// Work in progress of a 2D game using glutin for context creation/input
// handling and gl-rs for OpenGL bindings. The game will be a simple top down
// action-RPG created for educational purposes to assess the viability of Rust
// as a video game development language.
//
// Brian Ho
// brian@brkho.com
// December 2015

// #[macro_use]
// extern crate mmo;
extern crate cgmath;
// extern crate glutin;
extern crate gl;
// extern crate time;

// use glutin::{Event, Window};

use cgmath::*;
use gl::types::*;
// use std::mem;
// use std::ptr;
// use std::str;
// use std::ffi::CString;
// use std::fs::File;
// use std::io::Read;
// use std::process;
// use mmo::util::bmp;

use std::rc::Rc;

// Aliasing of cgmath types for uniformity in the game engine.
pub type Vector3D = Vector3<GLfloat>;
pub type Vector4D = Vector4<GLfloat>;

// Represents a color in RGBA with intensity values from 0 to 255.
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

// Currently unimplemented because I do not have a way of loading in COLLADA/fbx files.
pub struct Material { pub id: u8 }

// Stores information about the model which can be instantiated to create a ModelInstance. 
pub struct ModelInfo {
    pub vertices: Vec<GLfloat>,
    pub color: Color,
    pub mat: Material,
}

// An instantiazation of a ModelInfo that represents a model in-game. This has a variety of
// positional attributes used to render the instance.
pub struct ModelInstance {
    pub info: Rc<ModelInfo>,
    pub pos: Vector3D,
    pub scale: Vector3D,
    pub rot: Vector4D,
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
    pub fn new(pos: Vector3D, target: Vector3D, up: Vector3D, aspect: f32,
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
// around the glium Window class and will manage draws to the glium window.
pub struct GameWindow {
    pub width: u32,
    pub height: u32,
    pub title: String,
    pub bg_color: Color,
    pub camera: Option<Box<Camera>>,
    point_lights: Vec<PointLight>,
    directional_lights: Vec<DirectionalLight>,
    spot_lights: Vec<SpotLight>,
}

fn main() {
    println!("Hello, world!");
}
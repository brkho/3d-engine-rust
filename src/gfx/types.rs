extern crate cgmath;
extern crate gl;

pub use self::gl::types::*;
use std::os::raw;

// Redeclaration of the constant void pointer type for ease of use.
pub type CVoid = *const raw::c_void;

// Aliasing of cgmath types for uniformity in the game engine.
pub type Vector3D = cgmath::Vector3<GLfloat>;
pub type Quaternion = cgmath::Quaternion<GLfloat>;
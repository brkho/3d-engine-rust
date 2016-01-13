// Defines three types of lights and their associated methods. PointLight represents a light which
// emanates in all directions from a point. DirectionalLight represents a light which shines in
// one direction at a constant intensity (kind of like a PointLight from an infinite distance with
// with no attenuation). SpotLight is like a PointLight except it has a cutoff angle with a dropoff
// factor.
//
// Brian Ho
// brian@brkho.com

extern crate gl;

use gfx::color;
use gfx::types::*;
use std::ffi::CString;

// Light source that emanates from a fixed point with specified intensity and attenuation.
pub struct PointLight {
    pub intensity: color::Color,
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
    pub intensity: color::Color,
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
        uniform_vec3!(program, lights![li, "intensity"], color);
        uniform_vec3!(program, lights![li, "direction"], v3d_to_vec!(self.direction));
    }}
}

// Light source that emanates from a fixed point like a PointLight, but has a certain arc and
// falloff (like a flashlight).
pub struct SpotLight {
    pub intensity: color::Color,
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
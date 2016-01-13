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

// Light source that emanates from a fixed point with specified intensity and attenuation.
pub struct PointLight {
    pub intensity: color::Color,
    pub position: Vector3D,
    pub const_attn: f32,
    pub linear_attn: f32,
    pub quad_attn: f32,
    pub light_index: Option<usize>,
}

impl PointLight {
    // Default constructor for a PointLight.
    pub fn new(intensity: color::Color, position: Vector3D, const_attn: f32, linear_attn: f32,
            quad_attn: f32) -> PointLight {
        PointLight { intensity: intensity, position: position, const_attn: const_attn,
                linear_attn: linear_attn, quad_attn: quad_attn, light_index: None }
    }
}

// Light source that shines from an infinite distance from a direction (such as the sun).
pub struct DirectionalLight {
    pub intensity: color::Color,
    pub direction: Vector3D,
    pub light_index: Option<usize>,
}

impl DirectionalLight {
    // Default constructor for a DirectionalLight.
    pub fn new(intensity: color::Color, direction: Vector3D) -> DirectionalLight {
        DirectionalLight { intensity: intensity, direction: direction, light_index: None }
    }
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
    pub light_index: Option<usize>,
}

impl SpotLight {
    // Default constructor for a SpotLight.
    pub fn new(intensity: color::Color, position: Vector3D, direction: Vector3D, const_attn: f32,
            linear_attn: f32, quad_attn: f32, cutoff: f32, dropoff: f32) -> SpotLight {
        SpotLight { intensity: intensity, position: position, const_attn: const_attn,
                direction: direction, linear_attn: linear_attn, quad_attn: quad_attn,
                cutoff: cutoff, dropoff: dropoff, light_index: None }

    }
}
// Holds information about and defines a few ways to create a Material.
//
// Brian Ho
// brian@brkho.com

extern crate gl;

use gfx::color;
use gfx::types::*;
use std::mem;
use util::bmp;

// Describes a material for a model that contains a color, diffuse map, specular map, and a
// shininess factor for specular. This can only be created after the window context is set up.
pub struct Material {
    pub color: color::Color,
    pub diffuse: GLuint,
    pub specular: GLuint,
    pub normal: GLuint,
    pub shininess: GLfloat,
}

impl Material {
    // Default constructor that automatically assigns a white color given a shininess and paths to
    // the diffuse and specular maps as BMPs.
    pub fn new(diffuse_name: Option<String>, specular_name: Option<String>,
            normal_name: Option<String>, shininess: GLfloat) -> Material {
        Material::new_with_color(diffuse_name, specular_name, normal_name,
                color::Color::new_rgb(1.0, 1.0, 1.0), shininess)
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
            normal_name: Option<String>, color: color::Color, shininess: GLfloat) -> Material {
        let diffuse = Material::read_and_bind(diffuse_name);
        let specular = Material::read_and_bind(specular_name);
        // TODO: Just use the rgb vec.
        let normal = Material::read_and_bind(normal_name);
        Material { color: color, diffuse: diffuse, specular: specular, normal: normal,
                shininess: shininess }
    }
}
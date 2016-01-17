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
    pub normal: Option<GLuint>,
    pub shininess: GLfloat,
}

impl Material {
    // Default constructor that automatically assigns a white color given a shininess and paths to
    // the diffuse and specular maps as BMPs.
    pub fn new(diffuse_name: Option<&str>, specular_name: Option<&str>,
            normal_name: Option<&str>, shininess: GLfloat) -> Material {
        Material::new_with_color(diffuse_name, specular_name, normal_name,
                color::Color::new_rgb(1.0, 1.0, 1.0), shininess)
    }

    // Reads and binds a BMP texture given a name and returns the corresponding texture ID. This
    // method also lets the caller specify if the texture should be in sRGB space or not.
    fn read_and_bind(texture_name: Option<&str>, srgb: bool) -> GLuint { unsafe {
        if let Some(name) = texture_name {
            let texture = bmp::decode_bmp(name).unwrap().image;
            let image = texture.get_rgba_vec();
            let mut texture_id = 0;
            gl::GenTextures(1, &mut texture_id);
            gl::BindTexture(gl::TEXTURE_2D, texture_id);
            let color_space = if srgb { gl::SRGB_ALPHA } else { gl::RGBA };
            gl::TexImage2D(
                    gl::TEXTURE_2D, 0, color_space as GLsizei, texture.width as GLsizei,
                    texture.height as GLint, 0, gl::RGBA as GLuint, gl::UNSIGNED_BYTE,
                    vec_to_addr!(image));
            gl::GenerateMipmap(gl::TEXTURE_2D);
            texture_id
        } else { 0 }
    }}

    // Creates a Material with paths to diffuse and specular maps, shiniess, and color.
    pub fn new_with_color(diffuse_name: Option<&str>, specular_name: Option<&str>,
            normal_name: Option<&str>, color: color::Color, shininess: GLfloat) -> Material {
        let diffuse = Material::read_and_bind(diffuse_name, true);
        let specular = Material::read_and_bind(specular_name, false);
        // TODO: Just use the rgb vec.
        let normal = match normal_name {
            Some(_) => Some(Material::read_and_bind(normal_name, false)),
            None => None,
        };
        Material { color: color, diffuse: diffuse, specular: specular, normal: normal,
                shininess: shininess }
    }
}
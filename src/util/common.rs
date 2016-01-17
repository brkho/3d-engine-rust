// Common classes shared between the utility modules.
// 
// Brian Ho
// brian@brkho.com


extern crate cgmath;
extern crate gl;

use self::cgmath::*;
use self::gl::types::*;

// Defines what is in a vertex.
pub struct Vertex {
    pub pos: Vector3<GLfloat>,
    pub norm: Vector3<GLfloat>,
    pub tc: Vector2<GLfloat>,
    pub bitangent: Vector3<GLfloat>,
    pub tangent: Vector3<GLfloat>,
}

// A pixel with color and alpha information in the range 0-255.
pub struct Pixel {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
    pub alpha: u8,
}

// Defines what is in an image.
pub struct Image {
    pub width: u32,
    pub height: u32,
    pub data: Vec<Pixel>,
}

impl Image {
    // Helper method that refactors some of the code common to RGB and RGBA vector creation.
    fn get_vec_helper(&self, rgba: bool) -> Vec<u8> {
        let mut bmp_data = Vec::new();
        for pixel in &self.data {
            bmp_data.push(pixel.red);
            bmp_data.push(pixel.green);
            bmp_data.push(pixel.blue);
            if rgba {
                bmp_data.push(pixel.alpha);
            }
        }
        bmp_data
    }

    // Creates a one dimensional vector representing the BMP data with RGB channels.
    pub fn get_rgb_vec(&self) -> Vec<u8> {
        self.get_vec_helper(false)
    }

    // Creates a one dimensional vector representing the BMP data with RGBA channels.
    pub fn get_rgba_vec(&self) -> Vec<u8> {
        self.get_vec_helper(true)
    }
}
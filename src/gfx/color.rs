// Holds information about and defines a few ways to create a color in RGBA.
//
// Brian Ho
// brian@brkho.com

use gfx::types::*;

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
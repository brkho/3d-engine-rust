// Utility module that allows for decoding of a .rmod file given a path to a file. The .rmod file
// format is a binary file format native to the Rust game engine and can be created from a FBX file
// and texture maps using the rmod_converter.py script.
//
// Brian Ho
// brian@brkho.com

extern crate cgmath;
extern crate gl;

use self::cgmath::*;
use self::gl::types::*;
use std::fs::File;
use std::io::Read;
use util::common;

// Return value for a decoded BMP file. This contains a width, height, and an array of pixels with
// color and alpha information.
pub struct DecodedRMOD {
    pub diffuse: Option<common::Image>,
    pub specular: Option<common::Image>,
    pub normal: Option<common::Image>,
    pub vertices: Vec<common::Vertex>,
    pub elements: Vec<u32>,
    pub shininess: GLfloat,
}

// Handy constant for the number of bits in a byte (to avoid magic numbers in the code).
const BITS_PER_BYTE: usize = 8;

// Magic header as a quick way to verify that the file being loaded is actually .rmod format. This
// is like ELFMAGIC, but it stores RUSTGAME instead.
static RUSTGAME_MAGIC: [u8; 8] = [82, 85, 83, 84, 71, 65, 77, 69];

// Static array used for lookup for masking the nth bit in a byte.
static BIT_MASK: [u8; BITS_PER_BYTE] = [128, 64, 32, 16, 8, 4, 2, 1];

// Consumes n bytes from the byte vector by advancing the cursor while also performing error
// checking to see if we remain in bounds.
fn consume_n(data: &Vec<u8>, cursor: &mut usize, n: usize) -> Result<(), String> {
    let new_cursor = *cursor + n;
    if new_cursor > data.len() * BITS_PER_BYTE {
        return Err("RMOD file is too small.".to_string());
    }
    *cursor = new_cursor;
    Ok(())
}

// Reads a single bit, advances the cursor, and returns true if 1, else false.
fn read_bit(data: &Vec<u8>, cursor: &mut usize) -> Result<bool, String> {
    let orig = *cursor;
    try!(consume_n(data, cursor, 1));
    let byte = data[orig / BITS_PER_BYTE];
    let bit = byte & BIT_MASK[(orig % BITS_PER_BYTE)];
    return Ok(if bit == 0 { false } else { true })
}

// Reads n bits from the byte vector and returns it as an unsigned 32 bit integer. This is a bit
// inefficient because sometimes we want to read less than 32 bits and immediately cast to a
// smaller type like u8, but oh well.
fn read_n_bits(data: &Vec<u8>, cursor: &mut usize, n: usize) -> Result<u32, String> {
    if n > 32 { return Err("Too many bits to read at once.".to_string()); }
    let mut result: u32 = 0;
    for _ in 0..n {
        let value = if try!(read_bit(data, cursor)) { 1 } else { 0 };
        result = (result * 2) + value;
    }
    Ok(result)
}

// Reads a 32 bit signed float (IEEE 754) from the byte vector and returns it.
fn read_f32(data: &Vec<u8>, cursor: &mut usize) -> Result<f32, String> {
    let sign = if try!(read_bit(data, cursor)) { -1 } else { 1 };
    let exponent = try!(read_n_bits(data, cursor, 8)) as i32 - 127;
    let mut mantissa = 1.0;
    let mut divisor = 1.0;
    let mut changed = false;
    for _ in 0..23 {
        let value = try!(read_bit(data, cursor));
        if value { changed = true; }
        divisor /= 2.0;
        mantissa += (if value { 1 } else { 0 }) as f32 * divisor;
    }
    // Special case for 0.
    if !changed && exponent == -127 {
        Ok(sign as f32 * 0.0)
    } else {
        Ok(sign as f32 * mantissa * (2.0 as f32).powf(exponent as f32))
    }
}

// Reads a byte from the byte vector and returns it. This requires bit alignment unlike the other
// methods for efficiency purposes.
fn read_byte(data: &Vec<u8>, cursor: &mut usize) -> Result<u8, String> {
    let orig = *cursor;
    if orig % BITS_PER_BYTE != 0 {
        return Err("Cannot read unaligned byte.".to_string());
    }
    try!(consume_n(data, cursor, 8));
    Ok(data[orig / BITS_PER_BYTE])
}

// Reads a 32 bit unsigned integer from the byte vector and returns it.
fn read_u32(data: &Vec<u8>, cursor: &mut usize) -> Result<u32, String> {
    read_n_bits(data, cursor, 32)
}

// Verifies that the header of the file starts with the ASCII characters "RUSTGAME".
fn read_magic_header(data: &Vec<u8>, cursor: &mut usize) -> Result<(), String> {
    for i in 0..8 {
        let byte = try!(read_byte(data, cursor));
        if byte != RUSTGAME_MAGIC[i] { return Err("Magic header is invalid.".to_string()); }
    }
    Ok(())
}

// Reads in an image from the byte vector and returns an Image struct (or None if empty).
fn read_image(data: &Vec<u8>, cursor: &mut usize) -> Result<Option<common::Image>, String> {
    let width = try!(read_u32(data, cursor));
    let height = try!(read_u32(data, cursor));
    if width == 0 || height == 0 { return Ok(None); }
    let mut pixels: Vec<common::Pixel> = Vec::new();
    for _ in 0..(width * height) {
        let r = try!(read_byte(data, cursor));
        let g = try!(read_byte(data, cursor));
        let b = try!(read_byte(data, cursor));
        let a = try!(read_byte(data, cursor));
        pixels.push(common::Pixel { red: r, green: g, blue: b, alpha: a });
    }
    let image = common::Image { width: width, height: height, data: pixels };
    Ok(Some(image))
}

// Reads in a 3 dimensional vector from the byte vector and returns a Vector3 struct.
fn read_vec3(data: &Vec<u8>, cursor: &mut usize) -> Result<Vector3<GLfloat>, String> {
    let v1 = try!(read_f32(data, cursor));
    let v2 = try!(read_f32(data, cursor));
    let v3 = try!(read_f32(data, cursor));
    Ok(Vector3::new(v1, v2, v3))
}

// Reads in a 2 dimensional vector from the byte vector and returns a Vector3 struct.
fn read_vec2(data: &Vec<u8>, cursor: &mut usize) -> Result<Vector2<GLfloat>, String> {
    let v1 = try!(read_f32(data, cursor));
    let v2 = try!(read_f32(data, cursor));
    Ok(Vector2::new(v1, v2))
}

// Reads in a vertex from the byte vector and returns a Vertex struct.
fn read_vertex(data: &Vec<u8>, cursor: &mut usize) -> Result<common::Vertex, String> {
    let position = try!(read_vec3(data, cursor));
    let normal = try!(read_vec3(data, cursor));
    let tangent = try!(read_vec3(data, cursor));
    let bitangent = try!(read_vec3(data, cursor));
    let tcoord = try!(read_vec2(data, cursor));
    let vertex = common::Vertex { pos: position, norm: normal, tangent: tangent,
            bitangent: bitangent, tc: tcoord };
    Ok(vertex)
}

// Decodes a .rmod file given a path to the file and returns a DecodedRMOD struct containing the
// material and vertex information.
pub fn decode_rmod(fpath: &str) -> Result<DecodedRMOD, String> {
    let mut data = Vec::new();
    let mut cursor = 0;
    let mut fd = try!(File::open(fpath).map_err(|e| e.to_string()));
    try!(fd.read_to_end(&mut data).map_err(|e| e.to_string()));

    try!(read_magic_header(&data, &mut cursor));
    let diffuse = try!(read_image(&data, &mut cursor));
    let specular = try!(read_image(&data, &mut cursor));
    let normal = try!(read_image(&data, &mut cursor));
    let shininess = try!(read_f32(&data, &mut cursor));
    let mut vertices = Vec::new();
    let num_vertices = try!(read_u32(&data, &mut cursor));
    for _ in 0..num_vertices {
        vertices.push(try!(read_vertex(&data, &mut cursor)));
    }
    let mut elements = Vec::new();
    let num_elements = try!(read_u32(&data, &mut cursor));
    for _ in 0..num_elements {
        elements.push(try!(read_u32(&data, &mut cursor)));
    }
    if cursor != data.len() * BITS_PER_BYTE {
        return Err("RMOD file is improperly sized.".to_string());
    }
    let rmod_file = DecodedRMOD { diffuse: diffuse, specular: specular, normal: normal,
            vertices: vertices, elements: elements, shininess: shininess };
    Ok(rmod_file)
}


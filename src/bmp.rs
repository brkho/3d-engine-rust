// Utility module that allows for decoding of a BMP given a path to the file. This is only
// implemented for a very strict subset of possible BMP formats (BITMAPINFOHEADER) without
// compression. This is the format output by GIMP when exporting as BMP.
//
// Brian Ho
// brian@brkho.com
// December 2015


use std::fs::File;
use std::io::Read;
use std::mem;

// A pixel with color and alpha information in the range 0-255.
struct Pixel {
    red: u8,
    green: u8,
    blue: u8,
    alpha: u8,
}

// Data structure representation of the DIBHeader fields we care about.
struct DIBHeader {
    width: u32,
    height: u32,
    depth: u16,
}

// Return value for a decoded BMP file. This contains a width, height, and an array of pixels with
// color and alpha information.
struct DecodedBMP<'a> {
    width: u32,
    height: u32,
    data: &'a Vec<Pixel>,
}

// Consumes n bytes from the data vector by advancing the cursor while also performing error
// checking to see if we remain in bounds.
fn consume_n(data: &Vec<u8>, cursor: &mut usize, n: usize) -> Result<(), String> {
    let new_cursor = *cursor + n;
    if new_cursor > data.len() {
        return Err("BMP file is too small.".to_string());
    }
    *cursor = new_cursor;
    Ok(())
}

// Reads and consumes n bytes from the data vector and returns a slice of the data if successful.
fn read_n_bytes<'a>(data: &'a Vec<u8>, cursor: &mut usize, n: usize)
        -> Result<&'a [u8], String> {
    let orig = *cursor;
    try!(consume_n(data, cursor, n));
    Ok(&data[orig..(orig + n)])
}

// Reads and consumes 4 bytes from the data vector and casts the result to a u32.
fn read_dword(data: &Vec<u8>, cursor: &mut usize) -> Result<u32, String> {
    let bytes = try!(read_n_bytes(data, cursor, 4));
    let mut barr = [0; 4];
    for i in 0..4 {
        barr[i] = match bytes.get(i) {
            Some(v) => *v,
            None => return Err("Incorrect byte access.".to_string()),
        }
    }
    unsafe { Ok(mem::transmute::<[u8; 4], u32>(barr)) }
}

// Reads and consumes 2 bytes from the data vector and casts the result to a u16.
fn read_word(data: &Vec<u8>, cursor: &mut usize) -> Result<u16, String> {
    let bytes = try!(read_n_bytes(data, cursor, 2));
    let mut barr = [0; 2];
    for i in 0..2 {
        barr[i] = match bytes.get(i) {
            Some(v) => *v,
            None => return Err("Incorrect byte access.".to_string()),
        }
    }
    unsafe { Ok(mem::transmute::<[u8; 2], u16>(barr)) }
}

// Reads and consumes the initial BMP file header. This also performs the bare minimum amount of
// error checking by verifying that the first two bytes correspond to 'BM' in ASCII.
// TODO: Perform actual validation.
fn read_bmp_header(data: &Vec<u8>, cursor: &mut usize) -> Result<(), String> {
    let orig = *cursor;
    try!(consume_n(data, cursor, 14));
    if data[orig] != ('B' as u8) || data[orig + 1] != ('M' as u8) {
        return Err("BMP file header has incorrect magic values.".to_string())
    }
    Ok(())
}

// Reads and consumes the DIB header following the initial BMP file header. This uses helper
// functions to consume and read values from the DIB header to build a DIBHeader struct. We then
// return the constructed DIBHeader.
fn read_dib_header(data: &Vec<u8>, cursor: &mut usize) -> Result<DIBHeader, String> {
    let length = match try!(read_dword(data, cursor)) {
        l @ 40 | l @ 52 | l @ 56 | l @ 108 | l @ 124 => l, // Various BITMAPINFOHEADER versions.
        _ => return Err("Unsupported DIB header.".to_string()),
    };
    let width = try!(read_dword(data, cursor));
    let height = try!(read_dword(data, cursor));
    try!(consume_n(data, cursor, 2));
    let depth = try!(read_word(data, cursor));
    try!(consume_n(data, cursor, length as usize - 10));
    Ok(DIBHeader {width: width, height: height, depth: depth})
}

fn read_pixel_array<'a>(data: &Vec<u8>, cursor: &mut usize, info: &DIBHeader)
        -> Result<&'a Vec<Pixel>, String> {


    Err("hmm...".to_string())
}

// Decodes a BMP given a path to the file and returns a DecodedBMP struct containing the pixel
// information, width, and height of the image.
fn decode_bmp(fpath: &str) -> Result<DecodedBMP, String> {
    let mut data = Vec::new();
    let mut fd = try!(File::open(fpath).map_err(|e| e.to_string()));
    try!(fd.read_to_end(&mut data).map_err(|e| e.to_string()));

    let mut cursor = 0;
    try!(read_bmp_header(&data, &mut cursor));
    let info = try!(read_dib_header(&data, &mut cursor));
    let pixel_arr = try!(read_pixel_array(&data, &mut cursor, &info));
    println!("cursor: {}", cursor);



    Ok(DecodedBMP {width: info.width, height: info.height, data: pixel_arr})
}

// Driver test function.
fn main() {
    decode_bmp("test_texture.bmp").unwrap();
}
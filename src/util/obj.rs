// Utility module that allows for decoding of a OBJ file given a path to the file. This only
// supports a subset of the OBJ format at the current moment designed to work with the output of
// Maya 2015's OBJ exporter and popular CG meshes like the Stanford bunny. There is support for
// combinations of vertex, normal, and texture coord information.
//
// Brian Ho
// brian@brkho.com


use std::io::{BufRead, BufReader};
use std::fs::File;
use std::mem;


// Decodes an OBJ given a path to the file and returns a DecodedOBJ struct containing the vertex,
// normal, and texture coordinate info.
pub fn decode_obj(fpath: &str) -> Result<(), String> {
    let fd = try!(File::open(fpath).map_err(|e| e.to_string()));
    let reader = BufReader::new(&fd);
    for line_opt in reader.lines() {
        let line = line_opt.unwrap();
        let split: Vec<_> = line.split(char::is_whitespace).collect();
        if split.is_empty() { continue; }
        match split[0] {
            
        }
    }
    Ok(())
}


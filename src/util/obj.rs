// Utility module that allows for decoding of a OBJ file given a path to the file. This only
// supports a subset of the OBJ format at the current moment designed to work with the output of
// Maya 2015's OBJ exporter and popular CG meshes like the Stanford bunny. This parser assumes that
// the OBJ defines vertices, surface normals, and texture coordinates (like the standard exporter
// for Maya 2015). I could assign default texture coordinates and calculate normals from the cross
// product in their absence, but since this is a quick and dirty implementation, this is currently
// unsupported.
// 
// Brian Ho
// brian@brkho.com


extern crate cgmath;
extern crate gl;

use self::cgmath::*;
use self::gl::types::*;
use std::collections::{HashMap, HashSet};
use std::io::{BufRead, BufReader};
use std::fs::File;
use std::str::FromStr;
use util::common;

// The result of a OBJ decoding. This holds information about the vertices and elements.
pub struct DecodedOBJ {
    pub vertices: Vec<common::Vertex>,
    pub elements: Vec<(u32, u32, u32)>,
}

// Helper struct used to hold information about shared vertices and their shared normals, tangents,
// and bitangents.
struct SharedVertex {
    bitangent: Vector3<GLfloat>,
    tangent: Vector3<GLfloat>,
    vertices: HashSet<usize>,
}

// Process a vertex and return a Vector3 from its components.
fn process_vertex(info: &[&str]) -> Result<Vector3<GLfloat>, String> {
    let vertex = try!(process_float(info, "vertex", 3));
    // Change axis for engine compatibility from Maya 2015's output.
    Ok(Vector3::new(vertex[2], vertex[0], vertex[1]))
}

// Process a normal and return a Vector3 from its components.
fn process_normal(info: &[&str]) -> Result<Vector3<GLfloat>, String> {
    let normal = try!(process_float(info, "normal", 3));
    // Change axis for engine compatibility from Maya 2015's output.
    Ok(Vector3::new(normal[2], normal[0], normal[1]))
}

// Process a texture coordinate and return a Vector2 from its components.
fn process_tcoord(info: &[&str]) -> Result<Vector2<GLfloat>, String> {
    let tcoord = try!(process_float(info, "texture coordinate", 2));
    Ok(Vector2::new(tcoord[0], 1.0 - tcoord[1]))
}

// Helper function to refactor float processing with error checking.
fn process_float(info: &[&str], elem_type: &str, num: usize) -> Result<Vec<GLfloat>, String> {
    if info.len() != num {
        return Err(format!("A {} can only have {} components.", elem_type, num));
    }
    let mut result = Vec::new();
    for i in 0..num {
        result.push(try!(f32::from_str(info[i]).map_err(|e| e.to_string())));
    }
    Ok(result)
}

// Processes a face/texture/normal triplet with optional texture coordinates.
fn process_triplet(triplet: &str) -> Result<(u32, u32, u32), String> {
    let split: Vec<_> = triplet.split("/").collect();
    let face_err = "Invalid face declaration.".to_string();
    if split.len() != 3 {
        return Err(face_err);
    }
    let t1 = if split[0] == "" { None } else {
            Some(try!(u32::from_str(split[0]).map_err(|e| e.to_string()))) };
    let t2 = if split[1] == "" { None } else {
            Some(try!(u32::from_str(split[1]).map_err(|e| e.to_string()))) };
    let t3 = if split[2] == "" { None } else {
            Some(try!(u32::from_str(split[2]).map_err(|e| e.to_string()))) };
    let transformed = (t1, t2, t3);
    match transformed {
        (Some(v), Some(t), Some(n)) => Ok((v, t, n)),
        (Some(v), None, Some(n)) => Ok((v, 0, n)),
        _ => Err(face_err)
    }
}

// Process a triangle face and return a Vector3 from its components. This also calculates the
// tangent and bitangent based on a face and angle weighted average.
fn process_face(info: &[&str], vertices: &Vec<Vector3<GLfloat>>, normals: &Vec<Vector3<GLfloat>>,
        tcoords: &Vec<Vector2<GLfloat>>, vlist: &mut Vec<common::Vertex>,
        vmap: &mut HashMap<(u32, u32, u32), u32>, nmap: &mut HashMap<u32, SharedVertex>)
        -> Result<(u32, u32, u32), String> {
    if info.len() != 3 {
        return Err("The decoder only supports triangle meshes.".to_string());
    }
    let mut elems = Vec::new();
    let mut t_vertices = Vec::new();
    for i in 0..3 {
        let triplet = try!(process_triplet(info[i]));
        t_vertices.push(triplet.0);
        if !vmap.contains_key(&triplet) {
            // TODO: Make this code actually efficient and not just one giant hack with hashes.
            if !nmap.contains_key(&triplet.0) {
                let shared_vertex = SharedVertex {
                    bitangent: Vector3::new(0.0, 0.0, 0.0), tangent: Vector3::new(0.0, 0.0, 0.0),
                    vertices: HashSet::new() };
                nmap.insert(triplet.0, shared_vertex);
            }
            let mut shared_vertex = nmap.get_mut(&triplet.0).unwrap();
            shared_vertex.vertices.insert(vlist.len());

            let v = vertices[triplet.0 as usize - 1].clone();
            let t = if triplet.1 == 0 {
                    Vector2::new(0.0, 0.0) } else { tcoords[triplet.1 as usize - 1] }.clone();
            let n = normals[triplet.2 as usize - 1].clone();

            vmap.insert(triplet.clone(), vlist.len() as u32);
            vlist.push(common::Vertex { pos: v, tc: t, norm: n,
                    bitangent: Vector3::new(0.0, 0.0, 0.0),
                    tangent: Vector3::new(0.0, 0.0, 0.0) });
        }
        elems.push(vmap.get(&triplet).unwrap().clone());
    }

    let e1 = vlist[elems[1] as usize].pos - vlist[elems[0] as usize].pos;
    let e2 = vlist[elems[2] as usize].pos - vlist[elems[0] as usize].pos;
    let duv1 = vlist[elems[1] as usize].tc - vlist[elems[0] as usize].tc;
    let duv2 = vlist[elems[2] as usize].tc - vlist[elems[0] as usize].tc;

    let det = 1.0 / (duv1.x * duv2.y - duv1.y * duv2.x);
    let t1 = det * (duv2.y * e1.x - duv1.y * e2.x);
    let t2 = det * (duv2.y * e1.y - duv1.y * e2.y);
    let t3 = det * (duv2.y * e1.z - duv1.y * e2.z);
    let b1 = det * (-duv2.x * e1.x + duv1.x * e2.x);
    let b2 = det * (-duv2.x * e1.y + duv1.x * e2.y);
    let b3 = det * (-duv2.x * e1.z + duv1.x * e2.z);

    let tangent = Vector3::new(t1, t2, t3).normalize();
    let bitangent = Vector3::new(b1, b2, b3).normalize();

    let triangle_area = e1.cross(e2).length() * 0.5;
    // // Update vertex normals.
    for i in 0..3 {
        let mut shared_vertex = nmap.get_mut(&t_vertices[i]).unwrap();
        let new_tangent = shared_vertex.tangent + (tangent * triangle_area);
        let new_bitangent = shared_vertex.bitangent + (bitangent * triangle_area);
        shared_vertex.tangent = new_tangent;
        shared_vertex.bitangent = new_bitangent;
    }
    Ok((elems[0], elems[1], elems[2]))
}

// Decodes an OBJ given a path to the file and returns a DecodedOBJ struct containing the vertex,
// normal, and texture coordinate info.
pub fn decode_obj(fpath: &str) -> Result<DecodedOBJ, String> {
    let fd = try!(File::open(fpath).map_err(|e| e.to_string()));
    let reader = BufReader::new(&fd);
    let mut vertices: Vec<Vector3<GLfloat>> = Vec::new();
    let mut normals: Vec<Vector3<GLfloat>> = Vec::new();
    let mut tcoords: Vec<Vector2<GLfloat>> = Vec::new();
    let mut elements: Vec<(u32, u32, u32)> = Vec::new();
    let mut vlist: Vec<common::Vertex> = Vec::new();
    let mut vmap: HashMap<(u32, u32, u32), u32> = HashMap::new();
    let mut nmap: HashMap<u32, SharedVertex> = HashMap::new();
    for line_opt in reader.lines() {
        let line = line_opt.unwrap();
        let split: Vec<_> = line.split(char::is_whitespace).collect();
        if split.is_empty() { continue; }
        let key = split[0];
        let args = &split[1..];
        match key {
            "v" => { vertices.push(try!(process_vertex(args))) },
            "vt" => { tcoords.push(try!(process_tcoord(args))) },
            "vn" => { normals.push(try!(process_normal(args))) },
            "f" => {
                elements.push(try!(process_face(
                        args, &vertices, &normals, &tcoords, &mut vlist,
                        &mut vmap, &mut nmap))); },
            _ => (),
        }
    }
    for (_, shared_vertex) in nmap.iter() {
        let n_tangent = shared_vertex.tangent.normalize();
        let n_bitangent = shared_vertex.bitangent.normalize();
        for vid in shared_vertex.vertices.iter() {
            vlist[vid.clone()].tangent = n_tangent;
            vlist[vid.clone()].bitangent = n_bitangent;
        }
    }
    Ok(DecodedOBJ { vertices: vlist, elements: elements })
}


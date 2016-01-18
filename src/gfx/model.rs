// Defines various data structres relating to a model and its representation in 3D space. Broadly
// speaking, each model is first loaded in as a ModelInfo which contains vertex, normal, etc
// information and a BufferInfo. The BufferInfo stores information about where the ModelInfo's
// data is serialized in the GPU's memory. A ModelInfo can be used to create a ModelInfo which is
// a realizaton of the object in 3D space which can be rendered by the engine.
//
// Brian Ho
// brian@brkho.com

extern crate cgmath;

use self::cgmath::{Matrix, SquareMatrix};
use gfx::color;
use gfx::material;
use gfx::types::*;
use std::cell::Cell;
use std::rc::Rc;
use util::{common, obj, rmod};

#[derive(Copy, Clone)]
pub struct BufferInfo {
    pub gen: usize,
    pub start: usize,
    pub size: usize,
    pub vao: GLuint,
}

// Stores information about the model which can be instantiated to create a ModelInstance. 
pub struct ModelInfo {
    pub vertices: Vec<GLfloat>,
    pub normals: Vec<GLfloat>,
    pub bitangents: Vec<GLfloat>,
    pub tangents: Vec<GLfloat>,
    pub elements: Vec<GLuint>,
    pub tcoords: Vec<GLfloat>,
    pub mat: material::Material,
    pub buffer_info: Cell<Option<BufferInfo>>,
}

impl ModelInfo {
    // Default constructor with a material.
    pub fn new(vertices: Vec<GLfloat>, elems: Vec<GLuint>, normals: Vec<GLfloat>,
            tangents: Vec<GLfloat>, bitangents: Vec<GLfloat>, tcoords: Vec<GLfloat>,
            mat: material::Material) -> ModelInfo {
        ModelInfo { vertices: vertices, normals: normals, tangents: tangents,
                bitangents: bitangents, elements: elems, tcoords: tcoords, mat: mat,
                buffer_info: Cell::new(None) }
    }

    // Creates a box with specified size and color.
    pub fn new_box(scale_x: f32, scale_y: f32, scale_z: f32,
            mat: material::Material) -> ModelInfo {
        let vertices: Vec<GLfloat> = vec![
                -0.5 * scale_x, -0.5 * scale_y, -0.5 * scale_z,
                 0.5 * scale_x, -0.5 * scale_y, -0.5 * scale_z,
                 0.5 * scale_x,  0.5 * scale_y, -0.5 * scale_z,
                -0.5 * scale_x,  0.5 * scale_y, -0.5 * scale_z,
                -0.5 * scale_x, -0.5 * scale_y,  0.5 * scale_z,
                 0.5 * scale_x, -0.5 * scale_y,  0.5 * scale_z,
                 0.5 * scale_x,  0.5 * scale_y,  0.5 * scale_z,
                -0.5 * scale_x,  0.5 * scale_y,  0.5 * scale_z,
                -0.5 * scale_x,  0.5 * scale_y, -0.5 * scale_z,
        ];
        let elements: Vec<GLuint> = vec![
                0, 1, 2, 2, 3, 0, 4, 5, 6, 6, 7, 4, 7, 3, 0, 0, 4, 7, 6, 2, 1, 1, 5, 6, 0,
                1, 5, 5, 4, 0, 3, 2, 6, 6, 7, 8,
        ];
        let normals: Vec<GLfloat> = vec![0.0; 9 * 3];
        let tangents: Vec<GLfloat> = vec![0.0; 9 * 3];
        let bitangents: Vec<GLfloat> = vec![0.0; 9 * 3];
        let uvs: Vec<GLfloat> = vec![0.0; 9 * 2];
        ModelInfo::new(vertices, elements, normals, tangents, bitangents, uvs, mat)
    }

    // Helper method that refactors the lengthy code used to construct the data lists.
    fn vertex_to_data(vertices: &Vec<common::Vertex>) ->
            (Vec<GLfloat>, Vec<GLfloat>, Vec<GLfloat>, Vec<GLfloat>, Vec<GLfloat>) {
        let mut positions: Vec<GLfloat> = Vec::new();
        let mut normals: Vec<GLfloat> = Vec::new();
        let mut tangents: Vec<GLfloat> = Vec::new();
        let mut bitangents: Vec<GLfloat> = Vec::new();
        let mut tcoords: Vec<GLfloat> = Vec::new();
        for vertex in vertices {
            positions.push(vertex.pos.x);
            positions.push(vertex.pos.y);
            positions.push(vertex.pos.z);
            normals.push(vertex.norm.x);
            normals.push(vertex.norm.y);
            normals.push(vertex.norm.z);
            tangents.push(vertex.tangent.x);
            tangents.push(vertex.tangent.y);
            tangents.push(vertex.tangent.z);
            bitangents.push(vertex.bitangent.x);
            bitangents.push(vertex.bitangent.y);
            bitangents.push(vertex.bitangent.z);
            tcoords.push(vertex.tc.x);
            tcoords.push(vertex.tc.y);
        }
        (positions, normals, tangents, bitangents, tcoords)
    }

    // Helper function to create a ModelInfo and Material from an RMOD decoding with white color.
    pub fn from_rmod(rmod: &rmod::DecodedRMOD) -> ModelInfo {
        let color = color::Color::new_rgb(1.0, 1.0, 1.0);
        ModelInfo::from_rmod_color(rmod, color)
    }

    // Creates a ModelInfo with corresponding Material from the result of a RMOD decoding with a
    // specific color. Does a copy right now despite the inefficiency in order to avoid passing
    // ownership.
    pub fn from_rmod_color(rmod: &rmod::DecodedRMOD, color: color::Color) -> ModelInfo {
        let mat =  material::Material::from_images(&rmod.diffuse, &rmod.specular, &rmod.normal,
                color, rmod.shininess);
        let (verts, norms, tans, bitans, tcs) = ModelInfo::vertex_to_data(&rmod.vertices);
        let mut elems: Vec<GLuint> = Vec::new();
        for element in &rmod.elements {
            elems.push(element.clone());
        }
        ModelInfo::new(verts, elems, norms, tans, bitans, tcs, mat)
    }

    // Creates a ModelInfo from the result of a OBJ decoding.
    pub fn from_obj(object: &obj::DecodedOBJ, mat: material::Material) -> ModelInfo {
        let (verts, norms, tans, bitans, tcs) = ModelInfo::vertex_to_data(&object.vertices);
        let mut elems: Vec<GLuint> = Vec::new();
        for element in &object.elements {
            elems.push(element.0);
            elems.push(element.1);
            elems.push(element.2);
        }
        ModelInfo::new(verts, elems, norms, tans, bitans, tcs, mat)
    }

    // Gets a single vector representing the the ModelInfo in VBO format.
    pub fn get_vbo_format(&self) -> Vec<GLfloat> {
        let mut vertices: Vec<GLfloat> = Vec::new();
        let mut y = 0;
        for x in 0..self.vertices.len() {
            if x % 3 != 0 {
                continue;
            }
            vertices.push(self.vertices[x]);
            vertices.push(self.vertices[x + 1]);
            vertices.push(self.vertices[x + 2]);
            vertices.push(self.normals[x]);
            vertices.push(self.normals[x + 1]);
            vertices.push(self.normals[x + 2]);
            vertices.push(self.tangents[x]);
            vertices.push(self.tangents[x + 1]);
            vertices.push(self.tangents[x + 2]);
            vertices.push(self.bitangents[x]);
            vertices.push(self.bitangents[x + 1]);
            vertices.push(self.bitangents[x + 2]);
            vertices.push(self.tcoords[y]);
            vertices.push(self.tcoords[y + 1]);
            y += 2;
        }
        vertices
    }
}

// An instantiazation of a ModelInfo that represents a model in-game. This has a variety of
// positional attributes used to render the instance.
pub struct ModelInstance {
    pub info: Rc<ModelInfo>,
    pub pos: Vector3D,
    pub rot: Quaternion,
    pub scale: f32,
    pub model: cgmath::Matrix4<GLfloat>,
    pub normal: cgmath::Matrix4<GLfloat>,
}

impl ModelInstance {
    // Create an instance from a reference counted pointer to a ModelInfo struct.
    pub fn from(info: Rc<ModelInfo>) -> ModelInstance {
        let pos = Vector3D::new(0.0, 0.0, 0.0);
        let rot = Quaternion::new(1.0, 0.0, 0.0, 0.0);
        let scale = 1.0;
        let model = cgmath::Matrix4::from(cgmath::Decomposed {
                scale: scale, rot: rot, disp: pos });
        let norm = model.clone().invert().unwrap().transpose();
        ModelInstance { info: info, pos: pos, scale: scale, rot: rot, model: model, normal: norm }
    }

    // Updates the model and normal matrices. This must be called after any sequence of struct
    // field changes for the changes to appear in-world.
    pub fn update(&mut self) {
        let model = cgmath::Matrix4::from(cgmath::Decomposed {
                scale: self.scale, rot: self.rot, disp: self.pos });
        let normal = model.clone().invert().unwrap().transpose();
        self.model = model;
        self.normal = normal;
    }
}

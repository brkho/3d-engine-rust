// Defines the data structures and implementations for the GameWindow which is essentially the
// main component of the overall game engine. This uses glutin under the hood to create the OpenGL
// context. This might be moved to a "core" crate in the future if the GameWindow must also handle
// sophisitcated input, physics, or sound.
//
// Brian Ho
// brian@brkho.com

extern crate cgmath;
extern crate gl;
extern crate glutin;

use self::cgmath::{Matrix, Point};
pub use self::glutin::{ElementState, Event, VirtualKeyCode};

use gfx::camera;
use gfx::camera::Camera;
use gfx::color;
use gfx::light;
use gfx::model;
use gfx::types::*;
use util::shader;
use self::glutin::{Window, WindowBuilder};
use std::cmp;
use std::ffi::CString;
use std::mem;
use std::path;
use std::ptr;
use std::rc::Rc;

// Number of elements in a VBO or EBO.
const BUFFER_SIZE: usize = 65535 * 4;

// Maximum number of dynamic lights in a scene.
const MAX_LIGHTS: usize = 8;

// The default gamma of the scene.
const DEFAULT_GAMMA: GLfloat = 2.2;

// The default shader directory and names.
const SHADER_DIR: &'static str = "shaders";
const VERTEX_SHADER_NAME: &'static str = "std.vert";
const FRAGMENT_SHADER_NAME: &'static str = "std.frag";

// Contents of a VBO.
// [P_x  P_y  P_z  N_x  N_y  N_z  T_u  T_v]
const VERTEX_POS_SIZE: usize = 3;
const VERTEX_NORMAL_SIZE: usize = 3;
const VERTEX_TANGENT_SIZE: usize = 3;
const VERTEX_BITANGENT_SIZE: usize = 3;
const VERTEX_TCOORD_SIZE: usize = 2;
const VERTEX_SIZE: usize = VERTEX_POS_SIZE + VERTEX_NORMAL_SIZE + VERTEX_TANGENT_SIZE +
        VERTEX_BITANGENT_SIZE + VERTEX_TCOORD_SIZE;

// A window for graphics drawing that is managed by the graphics module. This is a thin wrapper
// around the glutin Window class and will manage draws to the glutin window.
pub struct GameWindow {
    pub bg_color: color::Color,
    pub cameras: Vec<Option<camera::PerspectiveCamera>>,
    pub program: GLuint,
    active_camera: Option<usize>,
    gl_window: Window,
    point_lights: Vec<Option<light::PointLight>>,               // (light_index, light)
    directional_lights: Vec<Option<light::DirectionalLight>>,   // (light_index, light)
    spot_lights: Vec<Option<light::SpotLight>>,                 // (light_index, light)
    light_indices: Vec<usize>,
    gen: usize,
    working_vao: GLuint,
    bound_vao: Option<GLuint>,
    default_texture: GLuint,
    gamma: GLfloat,
    vaos: Vec<Vec<Option<GLuint>>>,
    vbos: Vec<(GLuint, usize, usize)>, // (vbo_id, size, max_size)
    ebos: Vec<(GLuint, usize, usize)>, // (ebo_id, size, max_size)
}

impl GameWindow {
    // Initializes a GameWindow with a black background and no camera. Note that the GameWindow
    // creation can fail suchas unsupported OpenGL, so it returns a Result.
    pub fn new(width: u32, height: u32, title: String) -> Result<GameWindow, String> {
        let bg_color = color::Color::new_rgb(0.0, 0.0, 0.0);
        let pl: Vec<Option<light::PointLight>> = Vec::new();
        let dl: Vec<Option<light::DirectionalLight>> = Vec::new();
        let sl: Vec<Option<light::SpotLight>> = Vec::new();

        // TODO: Handle the actual error reporting of glutin and make this code less ugly.
        let creation_err = "Unable to create GameWindow.";
        let gl_window_builder = WindowBuilder::new().with_vsync().with_srgb(Some(true));
        let gl_window = try!(gl_window_builder.build().map_err(|_| creation_err.to_string()));
        unsafe { try!(gl_window.make_current().map_err(|_| creation_err.to_string())) }
        gl_window.set_title(&title);
        gl::load_with(|symbol| gl_window.get_proc_address(symbol) as *const _);
        let lights:Vec<usize> = (0..MAX_LIGHTS).collect();

        let mut window = GameWindow {
                bg_color: bg_color, cameras: Vec::new(), gl_window: gl_window,
                program: 0, point_lights: pl, directional_lights: dl, spot_lights: sl,
                active_camera: None, gen: 0, bound_vao: None, vbos: Vec::new(), ebos: Vec::new(),
                vaos: Vec::new(), working_vao: 0, light_indices: lights, default_texture: 0,
                gamma: 0.0 };

        // Begin unsafe OpenGL shenanigans. Here, we compile and link the shaders, set up the VAO
        // and VBO, and set some texture parameters.
        unsafe {
            let mut vpath = path::PathBuf::from(SHADER_DIR);
            vpath.push(VERTEX_SHADER_NAME);
            let mut fpath = path::PathBuf::from(SHADER_DIR);
            fpath.push(FRAGMENT_SHADER_NAME);
            let vs = shader::compile_shader(vpath.to_str().unwrap(), gl::VERTEX_SHADER);
            let fs = shader::compile_shader(fpath.to_str().unwrap(), gl::FRAGMENT_SHADER);
            window.program = shader::link_program(vs, fs);
            gl::GenVertexArrays(1, &mut window.working_vao);
            window.initialize_vbo(0);
            window.initialize_ebo(0);
            gl::Enable(gl::DEPTH_TEST);
            gl::UseProgram(window.program);
            gl::BindFragDataLocation(window.program, 0, gl_str!("out_color"));
            window.set_gamma(DEFAULT_GAMMA);

            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_BORDER as GLint);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_BORDER as GLint);
            gl::TexParameteri(
                    gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR_MIPMAP_NEAREST as GLint);
            gl::TexParameteri(
                    gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR_MIPMAP_NEAREST as GLint);
            // Set up the default white texture.
            let white_tex: Vec<u8> = vec![255, 255, 255];
            gl::GenTextures(1, &mut window.default_texture);
            gl::BindTexture(gl::TEXTURE_2D, window.default_texture);
            gl::TexImage2D(
                gl::TEXTURE_2D, 0, gl::SRGB as GLsizei, 1, 1, 0, gl::RGB as GLuint,
                gl::UNSIGNED_BYTE, vec_to_addr!(white_tex));
            gl::GenerateMipmap(gl::TEXTURE_2D);
            gl::BindTexture(gl::TEXTURE_2D, 0);
        }

        window.set_size(width, height);
        window.clear();
        window.swap_buffers();
        Ok(window)
    }

    // A helper method for binding the VAO and VBO that sets/checks the previously bound buffer.
    fn bind_vao_checked(&mut self, vao: GLuint) { unsafe {
        if match self.bound_vao {
                Some(bv) => bv != vao,
                None => true, } {
            self.bound_vao = Some(vao);
            gl::BindVertexArray(vao);
        }
    } }

    // Initializes a managed empty VBO of size BUFFER_SIZE and adds it to the vector of VBOs. This
    // also adds an uninitialized row to the VAOs data structure. This takes a max argument to
    // still not fail on creation even if we create a VBO for greater than BUFFER_SIZE elems.
    fn initialize_vbo(&mut self, max: usize) { unsafe {
        let buffer_size = cmp::max(max, BUFFER_SIZE);
        let working_vao = self.working_vao.clone();
        self.bind_vao_checked(working_vao);
        let mut vbo = 0;
        gl::GenBuffers(1, &mut vbo);
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::BufferData(
                gl::ARRAY_BUFFER, float_size!(buffer_size, GLsizeiptr),
                0 as CVoid, gl::STATIC_DRAW);
        self.vbos.push((vbo, 0, buffer_size));
        let size = if self.vaos.is_empty() { 0 } else { self.vaos[0].len() };
        self.vaos.push(vec![None; size]);
    }}

    // Initializes a managed empty EBO of size BUFFER_SIZE and adds it to the vector of EBOs. This
    // also adds an uninitialized column to the VAOs data structure. This takes a max argument to
    // still not fail on creation even if we create a EBO for greater than BUFFER_SIZE elems.
    fn initialize_ebo(&mut self, max: usize) { unsafe {
        let buffer_size = cmp::max(max, BUFFER_SIZE);
        let working_vao = self.working_vao.clone();
        self.bind_vao_checked(working_vao);
        let mut ebo = 0;
        gl::GenBuffers(1, &mut ebo);
        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
        gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER, uint_size!(buffer_size, GLsizeiptr),
                0 as CVoid, gl::STATIC_DRAW);
        self.ebos.push((ebo, 0, buffer_size));
        for vao_vec in self.vaos.iter_mut() {
            vao_vec.push(None);
        }
    }}

    // Initializes a VAO if there is not already an existing one for the EBO/VBO combination and
    // returns the corresponding VAO ID.
    fn initialize_vao(&mut self, vbo: usize, ebo: usize) -> GLuint { unsafe {
        match self.vaos[vbo as usize][ebo as usize] {
            Some(id) => id,
            None => {
                let mut vao = 0;
                gl::GenVertexArrays(1, &mut vao);
                self.bind_vao_checked(vao);
                self.vaos[vbo][ebo] = Some(vao);
                gl::BindBuffer(gl::ARRAY_BUFFER, self.vbos[vbo].0);
                gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.ebos[ebo].0);
                let pos_attr = gl::GetAttribLocation(self.program, gl_str!("position"));
                gl::EnableVertexAttribArray(pos_attr as GLuint);
                gl::VertexAttribPointer(
                        pos_attr as GLuint, VERTEX_POS_SIZE as i32, gl::FLOAT,
                        gl::FALSE as GLboolean, float_size!(VERTEX_SIZE, GLsizei), ptr::null());
                let normal_attr = gl::GetAttribLocation(self.program, gl_str!("normal"));
                gl::EnableVertexAttribArray(normal_attr as GLuint);
                gl::VertexAttribPointer(
                        normal_attr as GLuint, VERTEX_NORMAL_SIZE as i32, gl::FLOAT,
                        gl::FALSE as GLboolean, float_size!(VERTEX_SIZE, GLsizei),
                        float_size!(VERTEX_POS_SIZE, CVoid));

                let tangent_attr = gl::GetAttribLocation(self.program, gl_str!("tangent"));
                gl::EnableVertexAttribArray(tangent_attr as GLuint);
                gl::VertexAttribPointer(
                        tangent_attr as GLuint, VERTEX_TANGENT_SIZE as i32, gl::FLOAT,
                        gl::FALSE as GLboolean, float_size!(VERTEX_SIZE, GLsizei),
                        float_size!(VERTEX_POS_SIZE + VERTEX_NORMAL_SIZE, CVoid));

                let bitangent_attr = gl::GetAttribLocation(self.program, gl_str!("bitangent"));
                gl::EnableVertexAttribArray(bitangent_attr as GLuint);
                gl::VertexAttribPointer(
                        bitangent_attr as GLuint, VERTEX_BITANGENT_SIZE as i32, gl::FLOAT,
                        gl::FALSE as GLboolean, float_size!(VERTEX_SIZE, GLsizei),
                        float_size!(VERTEX_POS_SIZE + VERTEX_NORMAL_SIZE + VERTEX_TANGENT_SIZE,
                        CVoid));

                let tcoord_attr = gl::GetAttribLocation(self.program, gl_str!("tcoord"));
                gl::EnableVertexAttribArray(tcoord_attr as GLuint);
                gl::VertexAttribPointer(
                        tcoord_attr as GLuint, VERTEX_TCOORD_SIZE as i32, gl::FLOAT,
                        gl::FALSE as GLboolean, float_size!(VERTEX_SIZE, GLsizei),
                        float_size!(VERTEX_POS_SIZE + VERTEX_NORMAL_SIZE + VERTEX_TANGENT_SIZE
                        + VERTEX_BITANGENT_SIZE, CVoid));
                vao
            },
        }
    }}

    // Sets the gamma of the context.
    pub fn set_gamma(&mut self, gamma: GLfloat) { unsafe {
        self.gamma = gamma;
        uniform_float!(self.program, "gamma", gamma);
    }}

    // Adds a Camera to the engine and returns an integer handle to that camera that can be used
    // with get_camera() and detach_camera().
    pub fn attach_camera(&mut self, camera: camera::PerspectiveCamera) -> usize {
        let mut index = None;
        for (i, elem) in self.cameras.iter().enumerate() {
            match elem {
                &None => { index = Some(i); },
                &Some(_) => (),
            }
        }
        let handle = match index {
            None => {
                self.cameras.push(Some(camera));
                self.cameras.len() - 1
            }
            Some(i) => {
                self.cameras[i] = Some(camera);
                i
            }
        };
        self.update_camera(handle);
        handle
    }

    // Updates the camera view matrix and the view uniform on the GPU. This must be called after
    // any sequence of struct field changes for the changes to appear in-world.
    pub fn update_camera(&mut self, handle: usize) {
        let program = self.program.clone();
        let camera = self.get_camera_mut(handle).unwrap();
        camera.view = cgmath::Matrix4::look_at(
                cgmath::Point3::from_vec(camera.pos),
                cgmath::Point3::from_vec(camera.target),
                camera.up);
        unsafe { uniform_vec3!(program, "camera", v3d_to_vec!(camera.pos)) };
    }

    // Helper method that updates the active camera by calling update_camera().
    pub fn update_active_camera(&mut self) {
        let active_camera = match self.active_camera {
            None => { return; },
            Some(i) => i };
        self.update_camera(active_camera);
    }

    // Removes a camera from the engine and returns a Result if it was successful.
    pub fn detach_camera(&mut self, handle: usize) -> Result<(), String> {
        if handle >= self.cameras.len() { return Err("Out of range.".to_string()); }
        match self.active_camera {
            None => (),
            Some(c) => if handle == c { self.active_camera = None; },
        }
        self.cameras[handle] = None;
        Ok(())
    }

    // Takes in a handle and returns a mutable reference to the corresponding camera if it is
    // within range. Otherwise, return an Err.
    pub fn get_camera_mut(&mut self, handle: usize)
            -> Result<&mut camera::PerspectiveCamera, String> {
        if handle >= self.cameras.len() { return Err("Out of range.".to_string()); }
        Ok(self.cameras[handle].as_mut().unwrap())
    }

    // Takes in a handle and returns an immutable reference to the corresponding camera if it is
    // within range. Otherwise, return an Err.
    pub fn get_camera(&self, handle: usize) -> Result<&camera::PerspectiveCamera, String> {
        if handle >= self.cameras.len() { return Err("Out of range.".to_string()); }
        Ok(self.cameras[handle].as_ref().unwrap())
    }

    // Gets a mutable reference to the active camera. Returns Err if no current active camera.
    pub fn get_active_camera_mut(&mut self) -> Result<&mut camera::PerspectiveCamera, String> {
        match self.active_camera {
            None => Err("No currently active camera.".to_string()),
            Some(c) => self.get_camera_mut(c)
        }
    }

    // Gets an immutable reference to the active camera. Returns Err if no current active camera.
    pub fn get_active_camera(&self) -> Result<&camera::PerspectiveCamera, String> {
        match self.active_camera {
            None => Err("No currently active camera.".to_string()),
            Some(c) => self.get_camera(c)
        }
    }

    // Sets the active camera used for rendering given a handle.
    pub fn set_active_camera(&mut self, handle: usize) -> Result<(), String> {
        if handle >= self.cameras.len() { return Err("Out of range.".to_string()); }
        self.active_camera = Some(handle);
        Ok(())
    }

    // Attaches and transfers ownership of a point light to the window. This then returns a handle
    // (internally representing the index in the array) that can be used with the getter to modify
    // light attrs.
    pub fn attach_point_light(&mut self, mut light: light::PointLight) -> usize {
        light.light_index = self.light_indices.pop();
        let vec_index = GameWindow::add_light(light, &mut self.point_lights);
        self.update_point_light(vec_index);
        vec_index
    }

    // Updates the uniforms for a point light. This must be called after any sequence of struct
    // field changes for the changes to appear in-world.
    pub fn update_point_light(&self, index: usize) { unsafe {
        let light = self.get_point_light(index);
        let li = light.light_index.unwrap();
        uniform_uint!(self.program, lights![li, "type"], 1);
        let color = vec![light.intensity.r, light.intensity.g, light.intensity.b];
        uniform_vec3!(self.program, lights![li, "intensity"], color);
        uniform_vec3!(self.program, lights![li, "position"], v3d_to_vec!(light.position));
        uniform_float!(self.program, lights![li, "const_attn"], light.const_attn);
        uniform_float!(self.program, lights![li, "linear_attn"], light.linear_attn);
        uniform_float!(self.program, lights![li, "quad_attn"], light.quad_attn);
    }}

    // Removes a PointLight from the scene given its handle and returns it to transfer ownership.
    pub fn remove_point_light(&mut self, index: usize) -> light::PointLight {
        self.point_lights.push(None);
        let light = self.point_lights.swap_remove(index).unwrap();
        let free_index = light.light_index.unwrap();
        unsafe { uniform_uint!(self.program, lights![free_index, "type"], 0); };
        self.light_indices.push(free_index);
        light
    }

    // Gets a mutable reference to a PointLight given its handle.
    pub fn get_point_light_mut(&mut self, index: usize) -> &mut light::PointLight {
        (&mut self.point_lights[index]).as_mut().unwrap()
    }

    // Gets an immutable reference to a PointLight given its handle.
    pub fn get_point_light(&self, index: usize) -> &light::PointLight {
        (&self.point_lights[index]).as_ref().unwrap()
    }

    // Attaches and transfers ownership of a directional light to the window. This then returns a
    // handle (internally representing the index in the array) that can be used with the getter to
    // modify light attrs.
    pub fn attach_directional_light(&mut self, mut light: light::DirectionalLight) -> usize {
        light.light_index = self.light_indices.pop();
        let vec_index = GameWindow::add_light(light, &mut self.directional_lights);
        self.update_directional_light(vec_index);
        vec_index
    }

    // Updates the uniforms for a directional light. This must be called after any sequence of
    // struct field changes for the changes to appear in-world.
    pub fn update_directional_light(&self, index: usize) { unsafe {
        let light = self.get_directional_light(index);
        let li = light.light_index.unwrap();
        uniform_uint!(self.program, lights![li, "type"], 2);
        let color = vec![light.intensity.r, light.intensity.g, light.intensity.b];
        uniform_vec3!(self.program, lights![li, "intensity"], color);
        uniform_vec3!(self.program, lights![li, "direction"], v3d_to_vec!(light.direction));
    }}

    // Removes a DirectionalLight from the scene given its handle and returns it to transfer
    // ownership.
    pub fn remove_directional_light(&mut self, index: usize) -> light::DirectionalLight {
        self.directional_lights.push(None);
        let light = self.directional_lights.swap_remove(index).unwrap();
        let free_index = light.light_index.unwrap();
        unsafe { uniform_uint!(self.program, lights![free_index, "type"], 0); };
        self.light_indices.push(free_index);
        light
    }

    // Gets a reference to a DirectionalLight given its handle.
    pub fn get_directional_light_mut(&mut self, index: usize) -> &mut light::DirectionalLight {
        (&mut self.directional_lights[index]).as_mut().unwrap()
    }

    // Gets an immutable reference to a DirectionalLight given its handle.
    pub fn get_directional_light(&self, index: usize) -> &light::DirectionalLight {
        (&self.directional_lights[index]).as_ref().unwrap()
    }

    // Attaches and transfers ownership of a spot light to the window. This then returns a handle
    // (internally representing the index in the array) that can be used with the getter to modify
    // light attrs.
    pub fn attach_spot_light(&mut self, mut light: light::SpotLight) -> usize {
        light.light_index = self.light_indices.pop();
        let vec_index = GameWindow::add_light(light, &mut self.spot_lights);
        self.update_spot_light(vec_index);
        vec_index
    }

    // Updates the uniforms for a spot light. This must be called after any sequence of struct
    // field changes for the changes to appear in-world.
    pub fn update_spot_light(&self, index: usize) { unsafe {
        let light = self.get_spot_light(index);
        let li = light.light_index.unwrap();
        uniform_uint!(self.program, lights![li, "type"], 3);
        let color = vec![light.intensity.r, light.intensity.g, light.intensity.b];
        uniform_vec3!(self.program, lights![li, "intensity"], color);
        uniform_vec3!(self.program, lights![li, "position"], v3d_to_vec!(light.position));
        uniform_vec3!(self.program, lights![li, "direction"], v3d_to_vec!(light.direction));
        uniform_float!(self.program, lights![li, "const_attn"], light.const_attn);
        uniform_float!(self.program, lights![li, "linear_attn"], light.linear_attn);
        uniform_float!(self.program, lights![li, "quad_attn"], light.quad_attn);
        uniform_float!(self.program, lights![li, "cutoff"], light.cutoff);
        uniform_float!(self.program, lights![li, "dropoff"], light.dropoff);
    }}

    // Removes a SpotLight from the scene given its handle and returns it to transfer ownership.
    pub fn remove_spot_light(&mut self, index: usize) -> light::SpotLight {
        self.spot_lights.push(None);
        let light = self.spot_lights.swap_remove(index).unwrap();
        let free_index = light.light_index.unwrap();
        unsafe { uniform_uint!(self.program, lights![free_index, "type"], 0); };
        self.light_indices.push(free_index);
        light
    }

    // Gets a mutable reference to a SpotLight given its handle.
    pub fn get_spot_light_mut(&mut self, index: usize) -> &mut light::SpotLight {
        (&mut self.spot_lights[index]).as_mut().unwrap()
    }

    // Gets an immutable reference to a SpotLight given its handle.
    pub fn get_spot_light(&self, index: usize) -> &light::SpotLight {
        (&self.spot_lights[index]).as_ref().unwrap()
    }

    // Helper function that adds a light to a specified vector of lights. This keeps track of
    // "holes" in the array and returns a handle to the first unused location in the array. If
    // there are no holes, then it adds the light to the end and returns the corresponding handle.
    fn add_light<T>(light: T, vector: &mut Vec<Option<T>>) -> usize {
        let mut index = None;
        for (i, elem) in vector.iter().enumerate() {
            match elem {
                &None => { index = Some(i); },
                &Some(_) => (),
            }
        }
        match index {
            None => {
                vector.push(Some(light));
                vector.len() - 1
            }
            Some(i) => {
                vector[i] = Some(light);
                i
            }
        }
    }

    // Sets the size of the window.
    pub fn set_size(&self, width: u32, height: u32) {
        self.gl_window.set_inner_size(width, height);
    }

    // Gets the gl_window.
    pub fn poll_events(&self) -> glutin::PollEventsIterator {
        self.gl_window.poll_events()
    }

    // Clears the screen and buffers.
    pub fn clear(&self) {
        unsafe {
            gl::ClearColor(self.bg_color.r, self.bg_color.g, self.bg_color.b, self.bg_color.a);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }
    }

    // Swaps the buffers.
    pub fn swap_buffers(&self) {
        self.gl_window.swap_buffers().unwrap();
    }

    // Gets the size of the window in pixels. Again, just a poorly named wrapper. Sorry tomaka. :(
    pub fn get_size(&self) -> (u32, u32) {
        self.gl_window.get_inner_size_pixels().unwrap()
    }

    // Gets the aspect ratio of the window.
    pub fn get_aspect_ratio(&self) -> f32 {
        let (width, height) = self.get_size();
        (width as f32) / (height as f32)
    }

    // Maps/remaps a given Rc<ModelInfo> to VBO and EBO locations in the engine's managed buffers.
    pub fn map_vbo(&mut self, info: Rc<model::ModelInfo>) {
        let vertices = info.get_vbo_format();
        // Find empty EBO space.
        let ebo_index = {
            let mut index = None;
            for (i, ebo_pair) in self.ebos.iter().enumerate() {
                if info.elements.len() < ebo_pair.2 - ebo_pair.1 {
                    index = Some(i);
                    break;
                }
            }
            match index {
                None => {
                    self.initialize_ebo(info.elements.len() + 1);
                    self.ebos.len() - 1
                },
                Some(i) => i,
            }
        };

        // Find empty VBO space.
        let vbo_index = {
            let mut index = None;
            for (i, vbo_pair) in self.vbos.iter().enumerate() {
                if vertices.len() < vbo_pair.2 - vbo_pair.1 {
                    index = Some(i);
                    break;
                }
            }
            match index {
                None => {
                    self.initialize_vbo(vertices.len() + 1);
                    self.vbos.len() - 1
                },
                Some(i) => i,
            }
        };

        let vao = self.initialize_vao(vbo_index, ebo_index);
        let vbo_pair = self.vbos[vbo_index];
        let ebo_pair = self.ebos[ebo_index];

        let mut elements: Vec<GLuint> = Vec::new();
        for elem in &info.elements {
            elements.push(elem.clone() + (vbo_pair.1 as GLuint / VERTEX_SIZE as GLuint));
        }
        let buffer_info = model::BufferInfo {
                start: ebo_pair.1, size: elements.len(), gen: self.gen, vao: vao };
        info.buffer_info.set(Some(buffer_info));
        unsafe {
            let working_vao = self.working_vao.clone();
            self.bind_vao_checked(working_vao);
            gl::BufferSubData(
                    gl::ELEMENT_ARRAY_BUFFER, uint_size!(ebo_pair.1, GLintptr),
                    uint_size!(elements.len(), GLsizeiptr), vec_to_addr!(elements));
            gl::BufferSubData(
                    gl::ARRAY_BUFFER, float_size!(vbo_pair.1, GLintptr),
                    float_size!(vertices.len(), GLsizeiptr), vec_to_addr!(vertices));
        }
        self.ebos[ebo_index] = (ebo_pair.0, ebo_pair.1 + elements.len(), ebo_pair.2);
        self.vbos[vbo_index] = (vbo_pair.0, vbo_pair.1 + vertices.len(), vbo_pair.2);
    }

    // Clears the VBO/VAO/EBOs so that every ModelInfo currently mapped to the engine's VBO space
    // rmust be emapped on the next draw_instance() call.
    pub fn clear_vertex_buffers(&mut self) {
        let working_vao = self.working_vao.clone();
        self.bind_vao_checked(working_vao);
        self.gen += 1;
        for vbo_pair in self.vbos.iter_mut() {
            *vbo_pair = (vbo_pair.0, 0, vbo_pair.2);
        }
        for ebo_pair in self.ebos.iter_mut() {
            *ebo_pair = (ebo_pair.0, 0, ebo_pair.2);
        }
        for row in self.vaos.iter_mut() {
            for column in row.iter_mut() {
                if let Some(id) = *column {
                    let mut vao = id.clone();
                    unsafe { gl::DeleteVertexArrays(1, &mut vao); };
                    *column = None;
                }
            }
        }
    }

    // Draw a ModelInstance to the window using a camera, position, vertices, and materials.
    // This method also manages the engine's VBO space and updates the BufferInfo of the instance's
    // ModelInfo. If there is no associated BufferInfo for a ModelInfo, then we find an empty space
    // in the engine's VBO space and assign a new BufferInfo. If there is no more empty space in
    // any of the managed VBOs, we create a new VBO and assign it there instead. There is also a
    // generation field in both the BufferInfo and the Engine. On clear_vertex_buffers(), we
    // increment this generation count in the engine. If the generation count on the ModelInfo does
    // not match the count of the Engine, we remap.
    pub fn draw_instance(&mut self, instance: &model::ModelInstance) {
        match instance.info.buffer_info.get() {
            None => { self.map_vbo(instance.info.clone()); },
            Some(i) => { if i.gen != self.gen { self.map_vbo(instance.info.clone()) }; },
        }

        let transform = {
            let camera = match self.active_camera {
                None => { return; },
                Some(c) => self.cameras[c].as_ref().unwrap(),
            };
            let view = camera.get_view_matrix();
            let proj = camera.get_projection_matrix();
            proj * view * instance.model
        };

        unsafe {
            let mat = &instance.info.mat;
            let info = instance.info.buffer_info.get().unwrap();
            self.bind_vao_checked(info.vao);
            uniform_mat4!(self.program, "transform", transform);
            uniform_mat4!(self.program, "model", instance.model);
            uniform_mat4!(self.program, "normal_matrix", instance.normal);
            gl::ActiveTexture(gl::TEXTURE0);
            let diffuse_id = if mat.diffuse == 0 { self.default_texture } else { mat.diffuse };
            gl::BindTexture(gl::TEXTURE_2D, diffuse_id);
            uniform_int!(self.program, "diffuse_map", 0);
            gl::ActiveTexture(gl::TEXTURE1);
            let spec_id = if mat.specular == 0 { self.default_texture } else { mat.specular };
            gl::BindTexture(gl::TEXTURE_2D, spec_id);
            uniform_int!(self.program, "specular_map", 1);
            match mat.normal {
                Some(normal_id) => {
                    gl::ActiveTexture(gl::TEXTURE2);
                    gl::BindTexture(gl::TEXTURE_2D, normal_id);
                    uniform_int!(self.program, "normal_map", 2);
                    uniform_int!(self.program, "use_normal_map", 1); },
                None => { uniform_int!(self.program, "use_normal_map", 0); },
            };
            uniform_float!(self.program, "specular_coeff", mat.shininess);
            uniform_vec4!(self.program, "color", color_to_vec!(mat.color));

            gl::DrawElements(gl::TRIANGLES, info.size as i32,
                    gl::UNSIGNED_INT, uint_size!(info.start, CVoid));
        }
    }
}
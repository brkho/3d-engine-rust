// Defines various cameras for use in rendering. The base Camera trait specifies methods for
// getting the matrices needed for rendering. This also defines a PerspectiveCamera with associated
// methods. Future support is planned for adding an OrthographicCamera, but it is not currently
// implemented.
//
// Brian Ho
// brian@brkho.com

extern crate cgmath;
extern crate gl;

pub use self::cgmath::EuclideanVector;

use gfx::types::*;
use self::cgmath::SquareMatrix;

// Specifies methods for getting the view and projection matrices.
pub trait Camera {
    fn get_view_matrix(&self) -> cgmath::Matrix4<GLfloat>;
    fn get_projection_matrix(&self) -> cgmath::Matrix4<GLfloat>;
    fn get_fwd(&self) -> Vector3D;
    fn get_right(&self) -> Vector3D;
}

// A representation of a camera with a perspective projection. This implements the Camera trait, so
// it can be used as a camera for rendering the game.
pub struct PerspectiveCamera {
    pub pos: Vector3D,
    pub target: Vector3D,
    pub view: cgmath::Matrix4<GLfloat>,
    pub up: Vector3D,
    proj: cgmath::Matrix4<GLfloat>,
}

// Implementation of the Camera methods for PerspectiveCamera.
impl Camera for PerspectiveCamera {
    // Return the precomputed view matrix.
    fn get_view_matrix(&self) -> cgmath::Matrix4<GLfloat> { self.view }

    // Gets the forward vector from the view matrix.
    fn get_fwd(&self) -> Vector3D {
        let mat = self.get_view_matrix();
        Vector3D::new(mat.x[2], mat.y[2], mat.z[2]).normalize()
    }

    // Gets the right vector from the view matrix.
    fn get_right(&self) -> Vector3D {
        let mat = self.get_view_matrix();
        Vector3D::new(mat.x[0], mat.y[0], mat.z[0]).normalize()
    }

    // Return the precomputed projection matrix.
    fn get_projection_matrix(&self) -> cgmath::Matrix4<GLfloat> { self.proj }
}

// Implementation of PerspectiveCamera methods.
impl PerspectiveCamera {
    // Constructor to initialize the fields and set up the Projection matrix.
    pub fn new(pos: Vector3D, target: Vector3D, aspect: f32, fov: f32,
            near: f32, far: f32) -> PerspectiveCamera {
        let up = Vector3D::new(0.0, 0.0, 1.0);
        PerspectiveCamera::new_with_up(pos, target, up, aspect, fov, near, far)
    }

    // Constructor to initialize the fields and set up the Projection matrix with a specified up
    // vector.
    // TODO: Right now, you must call update directly after. I should factor this new code into the
    // GameWindow class like I do with lights.
    pub fn new_with_up(pos: Vector3D, target: Vector3D, up: Vector3D, aspect: f32,
            fov: f32, near: f32, far: f32) -> PerspectiveCamera {
        let proj = cgmath::PerspectiveFov {
                fovy: cgmath::Rad::from(cgmath::deg(fov)),
                aspect: aspect,
                near: near,
                far: far };
        let dummy_view = cgmath::Matrix4::identity();
        PerspectiveCamera { pos: pos, target: target,
                up: up, proj: cgmath::Matrix4::from(proj), view: dummy_view }
    }
}

// TODO: Write the OrthographicCamera.
// pub struct OrthographicCamera { }

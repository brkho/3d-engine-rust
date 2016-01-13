// Defines commonly used macros that should be shared across the crate. These are primarily used
// to make the code in gl-rs bindings cleaner and more sane. I'm honestly not sure I'm doing
// this macro thing correctly, but oh well.
//
// Brian Ho
// brian@brkho.com


#[macro_export]
// Macro for getting a multiple of the size for GLfloat and casting it as an OpenGL type.
macro_rules! float_size { ($n:expr, $t:ty) => (($n * mem::size_of::<GLfloat>()) as $t) }

#[macro_export]
// Macro for getting a multiple of the size for GLuint and casting it as an OpenGL type.
macro_rules! uint_size { ($n:expr, $t:ty) => (($n * mem::size_of::<GLuint>()) as $t) }

#[macro_export]
// Macro for casting and getting the address in memory of the first element of a vector.
macro_rules! vec_to_addr { ($i:expr) => (mem::transmute($i.get_unchecked(0))) }

#[macro_export]
// Macro for converting between &str and a GL readable string.
macro_rules! gl_str { ($s:expr) => (CString::new($s).unwrap().as_ptr()) }

#[macro_export]
// Macro for updating a mat4 uniform.
macro_rules! uniform_mat4 { ($p:expr, $s:expr, $l: expr) =>
        (gl::UniformMatrix4fv(
            gl::GetUniformLocation($p, gl_str!($s)), 1,
            gl::FALSE as GLboolean, ($l).as_ptr())) }

#[macro_export]
// Macro for updating a vec3 uniform.
macro_rules! uniform_vec3 { ($p:expr, $s:expr, $l: expr) =>
        (gl::Uniform3fv(
            gl::GetUniformLocation($p, gl_str!($s)), 1, ($l).as_ptr())) }

#[macro_export]
// Macro for updating a vec4 uniform.
macro_rules! uniform_vec4 { ($p:expr, $s:expr, $l: expr) =>
        (gl::Uniform4fv(
            gl::GetUniformLocation($p, gl_str!($s)), 1, ($l).as_ptr())) }

#[macro_export]
// Macro for updating a float uniform.
macro_rules! uniform_float { ($p:expr, $s:expr, $l: expr) =>
        (gl::Uniform1f(gl::GetUniformLocation($p, gl_str!($s)), $l)) }

#[macro_export]
// Macro for updating a uint uniform.
macro_rules! uniform_uint { ($p:expr, $s:expr, $l: expr) =>
        (gl::Uniform1ui(gl::GetUniformLocation($p, gl_str!($s)), $l)) }

#[macro_export]
// Macro for updating a int uniform.
macro_rules! uniform_int { ($p:expr, $s:expr, $l: expr) =>
        (gl::Uniform1i(gl::GetUniformLocation($p, gl_str!($s)), $l)) }

#[macro_export]
// Macro for getting a string representation of an index into the lights array
// uniform.
macro_rules! lights { [$i:expr, $f:expr] => (format!("lights[{}].{}", $i, $f)) }

#[macro_export]
// Macro for changing a Vector3D to vector of length 3.
macro_rules! v3d_to_vec { ($v:expr) => (vec![$v[0], $v[1], $v[2]]) }

#[macro_export]
// Macro for changing a Vector3D to vector of length 3.
macro_rules! color_to_vec { ($v:expr) => (vec![$v.r, $v.g, $v.b, $v.a]) }

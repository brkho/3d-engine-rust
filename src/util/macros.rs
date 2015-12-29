// Defines commonly used macros that should be shared across the crate. These are primarily used
// to make the code in gl-rs bindings cleaner and more sane. I'm honestly not sure I'm doing
// this macro thing correctly, but oh well.
//
// Brian Ho
// brian@brkho.com
// December 2015


#[macro_export]
// Macro for getting a multiple of the size for GLfloat and casting it as an OpenGL type.
macro_rules! float_size { ($n:expr, $t:ty) => (($n * mem::size_of::<GLfloat>()) as $t) }

#[macro_export]
// Macro for casting and getting the address in memory of the first element of a vector.
macro_rules! vec_to_addr { ($i:ident) => (mem::transmute($i.get_unchecked(0))) }

#[macro_export]
// Macro for converting between &str and a GL readable string.
macro_rules! gl_str { ($s:expr) => (CString::new($s).unwrap().as_ptr()) }

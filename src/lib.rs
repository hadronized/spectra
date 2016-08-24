extern crate gl;
pub extern crate glfw;
extern crate image;
extern crate luminance;
pub extern crate luminance_gl;
extern crate nalgebra;
// FIXME: use cfg(feature = "hot-reload")
extern crate notify;
extern crate openal;
extern crate vorbis;

#[macro_use]
pub mod report;
#[macro_use]
pub mod resource;

pub mod anim;
pub mod bootstrap;
pub mod color;
pub mod device;
pub mod entity;
pub mod objects;
pub mod projection;
pub mod shader;
pub mod texture;
pub mod transform;

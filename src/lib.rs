#![feature(associated_type_defaults)]
#![feature(proc_macro)]

extern crate gl;
pub extern crate glfw;
extern crate image;
pub extern crate luminance;
pub extern crate luminance_gl;
extern crate nalgebra;
#[cfg(feature = "hot-resource")]
extern crate notify;
extern crate openal;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
#[cfg(feature = "hot-resource")]
extern crate time;
extern crate vorbis;
extern crate wavefront_obj;

#[macro_use]
pub mod report;

#[macro_use]
pub mod resource;

pub mod anim;
pub mod app;
pub mod behavior;
pub mod bootstrap;
pub mod cache;
pub mod camera;
pub mod color;
pub mod device;
pub mod id;
pub mod model;
pub mod object;
pub mod objects; // FIXME: change the name of that module; it’s confusing
pub mod projection;
pub mod shader;
pub mod scene;
pub mod texture;
pub mod transform;

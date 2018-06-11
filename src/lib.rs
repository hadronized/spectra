#![feature(box_patterns)]
#![feature(box_syntax)]
#![feature(const_fn)]
#![feature(slice_patterns)]
#![feature(use_extern_macros)]

extern crate alto;
extern crate any_cache;
extern crate cgmath;
extern crate chrono;
extern crate clap;
extern crate image;
pub extern crate luminance;
extern crate luminance_glfw;
extern crate luminance_windowing;
extern crate glsl;
#[macro_use]
extern crate nom;
extern crate num_traits;
//extern crate rusttype; // FIXME: uncomment when we support text render back
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate vorbis;
extern crate wavefront_obj;
extern crate warmy;

// re-exported macros
pub use clap::{crate_authors, crate_name, crate_version};
pub use luminance::{gtup, uniform_interface, uniform_interface_build_struct,
                    uniform_interface_impl_trait, uniform_interface_impl_trait_map};

#[macro_use]
pub mod report;
#[macro_use]
pub mod sys;

pub mod audio;
//pub mod gui;
pub mod linear;
//pub mod overlay;
//pub mod text;

pub mod anim;
pub mod render;
pub mod scene;

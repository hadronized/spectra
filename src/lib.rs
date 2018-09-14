#![feature(box_patterns)]
#![feature(box_syntax)]
#![feature(slice_patterns)]

extern crate alto;
extern crate cgmath;
extern crate cheddar;
extern crate chrono;
extern crate clap;
extern crate image;
pub extern crate luminance;
extern crate luminance_glfw;
extern crate luminance_windowing;
extern crate num_traits;
extern crate serde;
#[macro_use] extern crate serde_derive;
extern crate serde_json;
extern crate splines;
extern crate vorbis;
extern crate wavefront_obj;
extern crate warmy;

// re-exported macros
pub use clap::{crate_authors, crate_name, crate_version};
pub use luminance::{gtup, uniform_interface, uniform_interface_build_struct,
                    uniform_interface_impl_trait, uniform_interface_impl_trait_map};

#[macro_use] pub mod report;
#[macro_use] pub mod sys;
pub mod anim;
pub mod audio;
pub mod linear;
pub mod render;
pub mod scene;

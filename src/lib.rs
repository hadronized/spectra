#![feature(associated_consts)]
#![feature(conservative_impl_trait)]
#![feature(const_fn)]

extern crate alto;
extern crate any_cache;
extern crate gl;
pub extern crate glfw;
extern crate image;
pub extern crate luminance;
extern crate nalgebra;
#[cfg(feature = "hot-resource")]
extern crate notify;
extern crate num;
extern crate rusttype;
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
#[macro_use]
pub mod scene;

pub mod anim;
pub mod audio;
pub mod app;
pub mod bootstrap;
pub mod camera;
pub mod compositor;
pub mod color;
pub mod edit;
pub mod extra;
//pub mod gui;
pub mod linear;
pub mod model;
pub mod object;
pub mod overlay;
pub mod projection;
pub mod renderer;
pub mod shader;
pub mod spline;
pub mod text;
pub mod texture;
pub mod transform;

pub use anim::Cont;
pub use audio::Audio;
pub use app::App;
pub use bootstrap::{Keyboard, Mouse, MouseMove, Scroll, WindowDim, bootstrap};
pub use camera::{Camera, Freefly};
pub use color::{Color, ColorAlpha};
pub use compositor::{Compositor, Screen};
pub use linear::{Matrix4};
pub use model::{Model, ModelError, Part};
pub use object::Object;
pub use projection::{Projectable, perspective};
pub use renderer::Renderer;
pub use resource::{Load, LoadError, Reload};
pub use shader::{Program, ShaderError, new_program};
pub use scene::Scene;
pub use spline::{Interpolate, Interpolation, Key, Spline, SplineIterator, Time};
pub use texture::{Texture, TextureImage, load_rgba_texture, save_rgba_texture};
pub use transform::{Axis, Orientation, Position, Translation, Transformable, X_AXIS, Y_AXIS, Z_AXIS,
                   Scale, translation_matrix};

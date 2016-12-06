#![feature(conservative_impl_trait)]
#![feature(const_fn)]
#![feature(proc_macro)]

extern crate gl;
pub extern crate glfw;
extern crate image;
pub extern crate luminance;
pub extern crate luminance_gl;
extern crate nalgebra;
#[cfg(feature = "hot-resource")]
extern crate notify;
extern crate num;
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
#[macro_use]
pub mod scene;

pub mod anim;
pub mod app;
pub mod bootstrap;
pub mod camera;
pub mod compositor;
pub mod color;
pub mod device;
pub mod extra;
pub mod id;
pub mod linear;
pub mod model;
pub mod object;
pub mod projection;
pub mod renderer;
pub mod shader;
pub mod spline;
pub mod texture;
pub mod transform;

pub use anim::Cont;
pub use app::App;
pub use bootstrap::{LuminanceBackend, Keyboard, Mouse, MouseMove, Scroll, WindowDim, bootstrap};
pub use camera::{Camera, Freefly};
pub use color::Color;
pub use compositor::{Compositor, Screen};
pub use device::Device;
pub use id::Id;
pub use linear::{Matrix4};
pub use model::{Model, ModelError, Part};
pub use object::Object;
pub use projection::{Projectable, perspective};
pub use renderer::Renderer;
pub use resource::{Load, LoadError, Reload};
pub use shader::{Program, ShaderError, new_program};
pub use scene::Scene;
pub use spline::{Interpolate, Interpolation, Key, Sampler, Spline, SplineIterator, Time};
pub use texture::{TextureImage, load_rgba_texture, save_rgba_texture};
pub use transform::{Axis, Orientation, Position, Translation, Transformable, X_AXIS, Y_AXIS, Z_AXIS,
                   Scale, translation_matrix};

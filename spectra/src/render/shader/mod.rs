//! Shaders are used within pipelines to consome vertices, geometry primitives and fragments.
//!
//! The `cheddar` module contains a GLSL-augmented language that enables the use of import statements
//! and analyses your code to switch on or off shader stages.

pub mod cheddar;
pub mod module;
pub mod program;

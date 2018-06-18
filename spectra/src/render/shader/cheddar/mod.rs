//! Shading language.
//!
//! This is an augmented GLSL that supports modules and deep analysis. That last feature makes it
//! possible to define a shader pipeline by composing several modules and use them in a single
//! module without even declaring a *vertex shader* or *fragment shader*: everything is analyzed and
//! shader stages are created from GLSL fragments.
//!
//! The `parser` module exports [nom](https://crates.io/crates/nom) parsers and `syntax` gives the
//! parsed AST.
pub mod parser;
pub mod syntax;

use luminance::framebuffer::Framebuffer;
use luminance::texture::{Dim2, Flat};

// Common framebuffer aliases.

pub type Framebuffer2D<C, D> = Framebuffer<Flat, Dim2, C, D>;

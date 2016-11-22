use luminance::Mode;
use luminance_gl::gl33::Tessellation;

/// A unit plane, aligned with the (x,y) plane.
pub fn new_plane() -> Tessellation {
  let vertices = [
    [ 1., -1., 0.],
    [ 1.,  1., 0.],
    [-1., -1., 0.],
    [-1.,  1., 0.]
  ];

  Tessellation::new(Mode::TriangleStrip, &vertices, None)
}

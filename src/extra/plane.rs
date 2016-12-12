use luminance::Mode;
use luminance_gl::gl33::Tess;

/// A unit plane, aligned with the (x,y) plane.
pub fn new_plane() -> Tess {
  let vertices = [
    [ 1., -1., 0.],
    [ 1.,  1., 0.],
    [-1., -1., 0.],
    [-1.,  1., 0.]
  ];

  Tess::new(Mode::TriangleStrip, &vertices, None)
}

use luminance::{Mode, Tess, TessVertices};

/// A unit plane, aligned with the (x,y) plane.
pub fn new_plane() -> Tess {
  let vertices = [
    [ 1., -1., 0.],
    [ 1.,  1., 0.],
    [-1., -1., 0.],
    [-1.,  1., 0.]
  ];

  Tess::new(Mode::TriangleStrip, TessVertices::Fill(&vertices), None)
}

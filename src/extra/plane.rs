use luminance::tess::{Mode, Tess, TessVertices};

/// A unit plane, aligned with the (x,y) plane.
pub fn new_plane() -> Tess<[f32; 3]> {
  let vertices = [
    [ 1., -1., 0.],
    [ 1.,  1., 0.],
    [-1., -1., 0.],
    [-1.,  1., 0.]
  ];

  Tess::new(Mode::TriangleStrip, TessVertices::Fill(&vertices), None)
}

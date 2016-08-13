use anim::{AnimParam, Key, Interpolation, Sampler};
use luminance::Mode;
use luminance_gl::gl33::Tessellation;

// A unit plane, aligned with the (x,y) plane.
pub fn new_plane() -> Tessellation {
  let vertices = [
    [ 1., -1., 0.],
    [ 1.,  1., 0.],
    [-1., -1., 0.],
    [-1.,  1., 0.]
  ];

  Tessellation::new(Mode::TriangleStrip, &vertices, None)
}

// A unit cube.
//
//     7-----5
//    /|    /|
//   3-+---1 |
//   | 6---+-4
//   |/    |/
//   2-----0
pub fn new_cube() -> Tessellation {
  let vertices = [
    [ 1., -1.,  1.],
    [ 1.,  1.,  1.],
    [-1., -1.,  1.],
    [-1.,  1.,  1.],
    [ 1., -1., -1.],
    [ 1.,  1., -1.],
    [-1., -1., -1.],
    [-1.,  1., -1.],
  ];

  let indices = [
    0, 1, 2, 2, 1, 3, // front face
    4, 5, 6, 6, 5, 7, // back face
    4, 5, 0, 0, 5, 1, // left face
    2, 3, 6, 6, 3, 7, // right face
    1, 5, 3, 3, 5, 7, // top face
    4, 0, 6, 6, 0, 2  // bottom face
  ];//

  Tessellation::new(Mode::Triangle, &vertices, Some(&indices))
}

// Build a curve connected by segments.
pub fn new_curve_2d(gap: f32, interpolation: Interpolation, points: &[(f32, f32)]) -> Tessellation {
  // convert 2D points into cps
  let cps = points.iter().map(|&(t, x)| Key::new(t, x, interpolation)).collect();
  let param = AnimParam::new(cps);

  let mut t = 0.;
  let mut sampler = Sampler::new();
  let mut vertices = Vec::new(); // FIXME: with_capacity ?

  // sample the curve
  while let Some(y) = sampler.sample(t, &param, true) {
    vertices.push([t, y]);
    t = t + gap;
  }

  Tessellation::new(Mode::LineStrip, &vertices, None)
}

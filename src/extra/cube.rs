use luminance::Mode;
use luminance_gl::gl33::Tessellation;

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
    1, 5, 3, 3, 5, 7, // top face
    2, 3, 6, 6, 3, 7, // right face
    4, 5, 0, 0, 5, 1, // left face
    4, 0, 6, 6, 0, 2, // bottom face
    4, 5, 6, 6, 5, 7, // back face
  ];

  Tessellation::new(Mode::Triangle, &vertices, Some(&indices))
}

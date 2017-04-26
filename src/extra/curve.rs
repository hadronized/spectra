use spline::{Spline, Key, Interpolation};
use luminance::tess::{Mode, Tess, TessVertices};

// Build a curve connected by segments.
pub fn new_curve_2d(gap: f32, interpolation: Interpolation, points: &[[f32; 2]]) -> Tess<[f32; 2]> {
  // convert 2D points into cps
  let cps = points.iter().map(|&[t, x]| Key::new(t, x, interpolation)).collect();
  let spline = Spline::from_keys(cps);

  let mut t = 0.;
  let mut vertices = Vec::new(); // FIXME: with_capacity ?

  // sample the curve
  while let Some(y) = spline.sample(t) {
    vertices.push([t, y]);
    t += gap;
  }

  Tess::new(Mode::LineStrip, TessVertices::Fill(&vertices), None)
}


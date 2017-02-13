use spline::{Spline, Key, Interpolation, Sampler};
use luminance::{Mode, Tess, TessVertices};

// Build a curve connected by segments.
pub fn new_curve_2d(gap: f32, interpolation: Interpolation, points: &[(f32, f32)]) -> Tess {
  // convert 2D points into cps
  let cps = points.iter().map(|&(t, x)| Key::new(t, x, interpolation)).collect();
  let spline = Spline::from_keys(cps);

  let mut t = 0.;
  let mut sampler = Sampler::new(&spline);
  let mut vertices = Vec::new(); // FIXME: with_capacity ?

  // sample the curve
  while let Some(y) = sampler.sample(t) {
    vertices.push([t, y]);
    t += gap;
  }

  Tess::new(Mode::LineStrip, TessVertices::Fill(&vertices), None)
}


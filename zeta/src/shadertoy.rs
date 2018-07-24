//! Shader Toy.

use spectra::anim::spline::{Interpolate, Spline};
use spectra::render::shader::{Uniform, Uniformable};
use spectra::sys::res::Res;
use spectra::sys::time::Time;

/// Animated parameter.
///
/// Such a parameter is used to represent a shader uniform that gets animated by a spline.
pub struct AnimatedParam<'a> {
  update_: Box<Fn(Time) + 'a>,
}

impl<'a> AnimatedParam<'a> {
  pub fn new<T, Q>(
    uniform: Uniform<Q>,
    spline: Res<Spline<T>>
  ) -> Self
  where T: 'a + Interpolate, Q: 'a + From<T> + Uniformable {
    Self {
      update_: Box::new(move |t| {
        if let Some(value) = spline.borrow().sample(t.as_f32()) {
          uniform.update(value.into());
        }
      })
    }
  }
}

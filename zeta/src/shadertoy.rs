//! Shader Toy.

use spectra::anim::spline::{Interpolate, Spline, SplineDeserializerAdapter};
use spectra::render::shader::{ProgramError, Uniform, UniformBuilder, UniformInterface, UniformWarning, Uniformable};
use spectra::sys::res::{FSKey, Res, Store};
use spectra::sys::time::Time;
use std::collections::{HashMap, HashSet};

/// Animated parameter.
///
/// Such a parameter is used to represent a shader uniform that gets animated by a spline.
pub struct AnimatedParam<'a> {
  update_: Box<Fn(Time) + 'a>,
}

impl<'a> AnimatedParam<'a> {
  pub fn new<T, Q, C>(
    builder: &UniformBuilder,
    store: &mut Store<C>,
    ctx: &mut C,
    name: &str
  ) -> Result<Self, ()>
  where T: 'static + SplineDeserializerAdapter + Interpolate, Q: 'a + From<T> + Uniformable {
    // first, get the spline
    let spline: Res<Spline<T>> =
      store.get(&FSKey::new(&format!("anim/parameters/{}.json", name)), ctx).map_err(|_| ())?; // FIXME

    // then, ask for the uniform
    let uniform = builder.ask::<Q>(name).map_err(|_| ())?; // FIXME

    let animated_param = Self {
      update_: Box::new(move |t| {
        if let Some(value) = spline.borrow().sample(t.as_f32()) {
          uniform.update(value.into());
        }
      })
    };

    Ok(animated_param)
  }
}

/// Set of animated params.
///
/// This is a shader uniform interface that will automatically update its parameters whenever they
/// change. In order 
pub struct AnimatedParams<'a> {
  params: HashMap<String, AnimatedParam<'a>>
}

struct AnimatedParamsContext<'a, C> where C: 'a {
  keys: HashSet<String>,
  store: &'a mut Store<C>,
  ctx: &'a mut C
}

// FIXME: for now, we just load 1D floating spline parameters
impl<'a, 'b, C> UniformInterface<AnimatedParamsContext<'b, C>> for AnimatedParams<'a> where C: 'b {
  fn uniform_interface(builder: &mut UniformBuilder, env: AnimatedParamsContext<C>) -> Result<Self, ProgramError> {
    let AnimatedParamsContext { keys, mut store, ctx } = env;

    // build the key-value iterator
    let kv = keys.into_iter().filter_map(move |k| {
      // try to create the animated parameter
      let ap = AnimatedParam::new::<f32, f32, C>(&builder, &mut store, ctx, &k);
      ap.map(|v| (k, v)).ok()
    });

    let params = AnimatedParams {
      params: kv.collect()
    };

    Ok(params)
  }
}

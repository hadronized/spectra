extern crate rand;
extern crate spectra;

use rand::{Rng, thread_rng};
use spectra::linear::{UnitQuaternion, Quaternion};
use spectra::spline::*;

#[test]
fn sampler_hold() {
  let mut sampler = Sampler::new();
  let p = Spline::new(vec![
    Key::new(0., 10., Interpolation::Step(1.)),
    Key::new(24., 100., Interpolation::Step(1.)),
    Key::new(45., -3.34, Interpolation::Step(1.))
  ]);

  assert_eq!(sampler.sample(0., &p, true), Some(10.));
  assert_eq!(sampler.sample(2., &p, true), Some(10.));
  assert_eq!(sampler.sample(23., &p, true), Some(10.));
  assert_eq!(sampler.sample(24., &p, true), Some(100.));
  assert_eq!(sampler.sample(44., &p, true), Some(100.));
  assert_eq!(sampler.sample(44., &p, false), Some(100.));
  assert_eq!(sampler.sample(45., &p, true), None);
  assert_eq!(sampler.sample(45347., &p, false), None);
  assert_eq!(sampler.sample(45347., &p, true), None);
}

#[test]
fn sampler_linear() {
  let mut sampler = Sampler::new();
  let p = Spline::new(vec![
    Key::new(0., 10., Interpolation::Linear),
    Key::new(10., 20., Interpolation::Linear)
  ]);

  assert_eq!(sampler.sample(0., &p, true), Some(10.));
  assert_eq!(sampler.sample(10., &p, true), None);
  assert_eq!(sampler.sample(5., &p, true), Some(15.));
}

#[test]
fn keys_sorted() {
  let nb = 10000;
  let mut rng = thread_rng();
  let mut cps = Vec::with_capacity(nb);

  for _ in 0..nb {
    let t = rng.gen::<f32>().abs();
    let v: f32 = rng.gen();
    let key = Key::new(t, v, Interpolation::Step(1.));

    cps.push(key);
  }

  let anim_param = Spline::new(cps);

  let mut t = 0.;
  for key in anim_param.into_iter() {
    assert!(t <= key.t, "t: {}, key.t: {}", t, key.t);
    t = key.t;
  }
}

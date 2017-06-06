extern crate rand;
extern crate spectra;

use rand::{Rng, thread_rng};
use spectra::linear::Quaternion;
use spectra::spline::*;

#[test]
fn hold() {
  let spline = Spline::from_keys(vec![
    Key::new(0., 10., Interpolation::Step(1.)),
    Key::new(24., 100., Interpolation::Step(1.)),
    Key::new(45., -3.34, Interpolation::Step(1.))
  ]);

  assert_eq!(spline.sample(0.), Some(10.));
  assert_eq!(spline.sample(2.), Some(10.));
  assert_eq!(spline.sample(23.), Some(10.));
  assert_eq!(spline.sample(24.), Some(100.));
  assert_eq!(spline.sample(44.), Some(100.));
  assert_eq!(spline.sample(44.), Some(100.));
  assert_eq!(spline.sample(45.), None);
  assert_eq!(spline.sample(45347.,), None);
  assert_eq!(spline.sample(45347.,), None);
}

#[test]
fn linear() {
  let spline = Spline::from_keys(vec![
    Key::new(0., 10., Interpolation::Linear),
    Key::new(10., 20., Interpolation::Linear)
  ]);

  assert_eq!(spline.sample(0.), Some(10.));
  assert_eq!(spline.sample(10.), None);
  assert_eq!(spline.sample(5.), Some(15.));
}

#[test]
fn keys_sorted() {
  let nb = 10000;
  let mut rng = thread_rng();
  let mut keys = Vec::with_capacity(nb);

  for _ in 0..nb {
    let t = rng.gen::<f32>().abs();
    let v: f32 = rng.gen();
    let key = Key::new(t, v, Interpolation::Step(1.));

    keys.push(key);
  }

  let anim_param = Spline::from_keys(keys);

  let mut t = 0.;
  for key in anim_param.into_iter() {
    assert!(t <= key.t, "t: {}, key.t: {}", t, key.t);
    t = key.t;
  }
}

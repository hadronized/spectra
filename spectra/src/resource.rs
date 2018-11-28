pub mod error;
pub mod key;

// /// Load helper.
// ///
// /// Call this function whenever you need to load a resource and that you want logged information,
// /// such as failures, timing, etc.
// pub fn load_with<T, A, E, F>(
//   path: &Path,
//   loader: F
// ) -> Result<A, E>
// where F: FnOnce() -> Result<A, E>,
//       T: TyDesc {
//   info!("loading {} {}", T::TY_DESC, path.display());
// 
//   let start_time = Instant::now();
//   let r = loader();
//   let t = start_time.elapsed();
//   let ns = t.as_secs() as f64 * 1e9 + t.subsec_nanos() as f64;
//   let (pretty_time, suffix) = load_time(ns);
// 
//   if let Ok(_) = r {
//     info!("loaded {} {}: {:.3}{}", T::TY_DESC, path.display(), pretty_time, suffix);
//   } else {
//     err!("fail to load {} {}: {:.3}{}", T::TY_DESC, path.display(), pretty_time, suffix);
//   }
// 
//   r
// }

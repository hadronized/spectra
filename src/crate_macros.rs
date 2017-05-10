#[macro_export]
macro_rules! crate_authors {
  () => { env!("CARGO_PKG_AUTHORS") }
}

#[macro_export]
macro_rules! crate_name {
  () => { env!("CARGO_PKG_NAME") }
}

#[macro_export]
macro_rules! crate_version {
  () => { env!("CARGO_PKG_VERSION") }
}

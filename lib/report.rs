#[macro_export]
macro_rules! deb {
  ( $e:expr ) => { println!(concat!("\x1b[36m       > ", $e, "\x1b[0m")); };
  ( $e:expr, $($arg:tt)* ) => { println!(concat!("\x1b[36m       > ", $e, "\x1b[0m"), $($arg)*); };
}

#[macro_export]
macro_rules! info {
  ( $e:expr ) => { println!(concat!("\x1b[34m       > ", $e, "\x1b[0m")); };
  ( $e:expr, $($arg:tt)* ) => { println!(concat!("\x1b[34m       > ", $e, "\x1b[0m"), $($arg)*); };
}

#[macro_export]
macro_rules! warn {
  ( $e:expr ) => { println!(concat!("\x1b[31m  /!\\  > ", $e, "\x1b[0m")); };
  ( $e:expr, $($arg:tt)* ) => { println!(concat!("\x1b[31m  /!\\ > ", $e, "\x1b[0m"), $($arg)*); };
}

#[macro_export]
macro_rules! err {
  ( $e:expr ) => { println!(concat!("\x1b[1;31m /!!!\\ > ", $e, "\x1b[0;0m")); };
  ( $e:expr, $($arg:tt)* ) => { println!(concat!("\x1b[1;31m /!!!\\ > ", $e, "\x1b[0;0m"), $($arg)*); };
}

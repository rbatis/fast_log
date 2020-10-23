//#![forbid(unsafe_code)]

#[macro_use]
extern crate lazy_static;
pub mod log;
pub mod error;
pub mod time_util;

///init log
pub use log::init_log;
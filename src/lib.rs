//#![forbid(unsafe_code)]

#[macro_use]
extern crate lazy_static;
pub mod fast_log;
pub mod error;
pub mod time_util;
pub mod plugin;

///init log
pub use fast_log::init_log as init_log;
pub use fast_log::init_custom_log as init_custom_log;
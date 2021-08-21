#![forbid(unsafe_code)]
#![allow(unused_variables)]
#![allow(unused_assignments)]
#![allow(unused_must_use)]

#[macro_use]
extern crate lazy_static;

pub mod appender;
pub mod bencher;
pub mod consts;
pub mod error;
pub mod fast_log;
pub mod filter;
pub mod plugin;
pub mod wait;

pub use fast_log::init_custom_log;
///init log
pub use fast_log::init_log;
pub use fast_log::init_split_log;

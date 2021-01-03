#![forbid(unsafe_code)]
#![allow(unused_variables)]
#![allow(unused_assignments)]
#![allow(unused_must_use)]

#[macro_use]
extern crate lazy_static;

mod bencher;
pub mod fast_log;
pub mod error;
pub mod plugin;
pub mod filter;
pub mod appender;
pub mod consts;

///init log
pub use fast_log::init_log as init_log;
pub use fast_log::init_split_log as init_split_log;
pub use fast_log::init_custom_log as init_custom_log;


///test
mod example;
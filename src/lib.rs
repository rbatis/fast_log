#![forbid(unsafe_code)]
#![allow(unused_variables)]
#![allow(unused_assignments)]
#![allow(unused_must_use)]

pub mod appender;
pub mod bencher;
pub mod consts;
pub mod error;
pub mod fast_log;
pub mod filter;
pub mod plugin;
pub mod wait;
pub mod runtime;
pub mod config;

pub use fast_log::*;
pub use runtime::*;
pub use crate::config::Config;


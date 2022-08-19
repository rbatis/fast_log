#![forbid(unsafe_code)]
#![allow(unused_variables)]
#![allow(unused_assignments)]
#![allow(unused_must_use)]

extern crate core;

pub mod appender;
pub mod bencher;
pub mod config;
pub mod consts;
pub mod error;
pub mod fast_log;
pub mod filter;
pub mod formats;
pub mod plugin;
pub mod runtime;

pub use crate::config::Config;
pub use crate::fast_log::*;
pub use crate::formats::*;
pub use runtime::*;

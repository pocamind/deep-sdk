#![warn(clippy::pedantic)]
#![allow(clippy::too_many_lines, clippy::missing_errors_doc)]

pub mod error;
pub mod model;
pub mod parse;
pub mod util;

pub use model::{data, req, stat::Stat};

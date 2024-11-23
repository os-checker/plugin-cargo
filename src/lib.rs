#[macro_use]
extern crate eyre;
#[macro_use]
extern crate tracing;

pub mod crates_io;
pub mod database;
pub mod nextest;
pub mod repo;

pub const BASE_DIR: &str = "cargo";

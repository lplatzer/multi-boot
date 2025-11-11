pub mod cli;
pub mod config;
pub mod device;
pub mod disk;
pub mod bootloader;
pub mod iso;
pub mod error;

pub use error::{MultiBootError, Result};

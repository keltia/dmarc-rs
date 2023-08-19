//! This crate implement the library part of `dmarc-rs`, dealing with IPs and list of IPs
//!

mod entry;
mod filetype;
mod res;
mod task;
mod types;

pub use entry::*;
pub use filetype::*;
pub use res::*;
pub use task::*;
pub use types::*;

//! This crate implement the library part of `dmarc-rs`, dealing with IPs and list of IPs
//!

pub use dmarc::*;
pub use entry::*;
pub use filetype::*;
pub use res::*;

mod dmarc;
mod entry;
mod filetype;
mod res;

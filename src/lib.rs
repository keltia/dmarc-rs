//! This crate implement the library part of `dmarc-rs`, dealing with IPs and list of IPs
//!

pub mod entry;
pub mod input;
pub mod res;
pub mod task;
pub mod types;

/// Simple macro to generate PathBuf from a series of entries
///
#[macro_export]
macro_rules! makepath {
    ($($item:expr),+) => {
        [
        $(PathBuf::from($item),)+
        ]
        .iter()
        .collect()
    };
}

/// Simple macro to generate PathBuf from a series of entries
///
#[macro_export]
macro_rules! makelist {
    ($($item:expr),+) => {
        vec![ $(PathBuf::from($item),)+ ]
    };
}

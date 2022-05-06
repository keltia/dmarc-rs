//! Main XML parser
//!
//!

// Std library
//
use std::fs::File;

// Our crates
//
use dmarc_rs::types::*;

// External crates
//
use anyhow::Result;

pub fn analyze_file(fh: &mut File) -> Result<String> {
    Ok("".into())
}

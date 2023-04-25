//! Main XML parser
//!
//!

use std::fmt::{Display, Formatter};
// Std library
//
use std::fs::File;
use std::path::PathBuf;

// Our crates
//
use dmarc_rs::types::*;

// External crates
//
use anyhow::Result;
use std::io::BufReader;

#[derive(Debug)]
pub struct Dmarc {
    pub fname: PathBuf,
    pub report: Feedback,
}

impl Dmarc {
    /// Decode the XML file and generate the report
    ///
    pub fn from_str(fname: PathBuf, data: &str) -> Result<Dmarc> {
        let rdr = BufReader::new(data);
        let report: Feedback = serde_xml_rs::from_reader(rdr)?;
        Ok(Dmarc { fname, report })
    }
}

impl Display for Dmarc {
    /// Generate the output through a template
    ///
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

//! Main XML parser
//!
//!

// Std library
//
use std::fmt::{Display, Formatter};
use std::io::BufReader;
use std::path::PathBuf;

// External crates
//
use eyre::Result;

// Our crates
//
use dmarc_rs::Feedback;

#[derive(Debug)]
pub struct Dmarc {
    pub fname: PathBuf,
    pub report: Feedback,
}

impl Dmarc {
    /// Decode the XML file and generate the report
    ///
    pub fn from_str(fname: PathBuf, data: &str) -> Result<Self> {
        let rdr = BufReader::new(data.as_bytes());
        let report: Feedback = serde_xml_rs::from_reader(rdr)?;
        Ok(Dmarc { fname, report })
    }
}

impl Display for Dmarc {
    /// Generate the output through a template
    ///
    fn fmt(&self, _f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

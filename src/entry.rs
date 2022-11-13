//! Entry is for storing a file type (based on its extension) along with the pathname.
//!
//! That way we can have a different function to manage Gzip archives, Zip ones, etc.
//!

// Std library
//
use std::fmt::{Debug, Formatter};
use std::fs::File;
use std::io::prelude::*;
use std::io::{BufReader, Read};
use std::path::PathBuf;

// Our crates
//
use crate::input::Input;

// Extra packages
//
use anyhow::{anyhow, Result};
use log::trace;

/// Entry carries the file path and its type (Xml, Gzip, etc.).
#[derive(Clone, Debug)]
pub struct Entry {
    /// Pathname if any, `<stdin>` otherwise
    pub p: PathBuf,
    /// File type as found by `Input::from_path(&str)` or through `-t`
    pub ft: Input,
    /// Result of the DMARC parsing
    pub res: String,
}

impl Default for Entry {
    #[inline]
    fn default() -> Self {
        Entry::new("")
    }
}

impl Entry {
    /// Create a new `Entry`  with the file type
    ///
    /// Example:
    /// ```
    /// use std::path::PathBuf;
    /// use dmarc_rs::entry::Entry;
    ///
    /// let f = Entry::new("Foo.zip");
    /// ```
    ///
    #[inline]
    pub fn new<P: Into<PathBuf>>(p: P) -> Self {
        let pp = p.into();
        Entry {
            p: pp.clone(),
            ft: Input::from_path(pp),
            res: "".to_owned(),
        }
    }

    /// Allow for changing the file type
    ///
    /// Example:
    /// ```
    /// use dmarc_rs::entry::Entry;
    /// use dmarc_rs::input::Input;
    ///
    /// // This is obviously wrong, don't do it :)
    /// let mut f = Entry::new("Foo.zip").set(Input::Gzip);
    /// ```
    ///
    #[inline]
    pub fn set(&mut self, t: Input) -> &mut Self {
        self.ft = t;
        self
    }

    /// Return the stored path
    ///
    #[inline]
    pub fn path(self) -> PathBuf {
        self.p
    }

    /// Return the Input type of the concerned entry
    ///
    #[inline]
    pub fn input_type(self) -> Input {
        self.ft
    }

    /// Open the given file and return the content as a String.
    ///
    /// This is where we call the different functions for the different types of
    /// input files.
    ///
    /// **NOTE** Plain files are assumed to be XML.
    ///
    /// Example:
    /// ```
    /// # use anyhow::anyhow;
    /// # use dmarc_rs::entry::Entry;
    /// let mut f = Entry::new("foo.xml");
    ///
    /// let xml = match f.fetch() {
    ///     Ok(s) => s,
    ///     Err(e) => anyhow!("Error reading.").to_string(),
    /// };
    /// ```
    ///
    pub fn get_data(self) -> Result<String> {
        match self.ft {
            Input::Csv | Input::Xml | Input::Plain => {
                let fh = match File::open(&self.p) {
                    Ok(fh) => fh,
                    Err(e) => return Err(anyhow!("{}", e.to_string())),
                };
                let mut s = String::new();
                BufReader::new(fh).read_to_string(&mut s)?;
                Ok(s)
            }
            Input::Zip => unimplemented!(),
            Input::Gzip => unimplemented!(),
        }
    }
}

pub fn read_zip(fh: &dyn BufRead) -> String {
    unimplemented!()
}

pub fn read_gzip(fh: &dyn BufRead) -> String {
    unimplemented!()
}

#[cfg(test)]
mod tests {
    use super::*;

    use anyhow::bail;
    use rstest::rstest;

    #[rstest]
    #[case("", Input::Xml)]
    #[case("foo", Input::Xml)]
    #[case("foo.zip", Input::Zip)]
    #[case("bar.gz", Input::Gzip)]
    #[case("baz.xml.gz", Input::Gzip)]
    fn test_new(#[case] p: &str, #[case] res: Input) {
        let e = Entry::new(&p);
        assert_eq!(res, e.input_type());
    }

    #[test]
    fn test_set() {
        let mut e = Entry::new("foo");

        e.set(Input::Gzip);
        assert_eq!(Input::Gzip, e.input_type());
    }

    #[test]
    fn test_entry_get_data() {
        let mut f = Entry::new("Cargo.toml");

        let txt = f.fetch();
        assert!(txt.is_ok());
        let txt = txt.unwrap();
        assert!(txt.contains("dmarc-rs"))
    }

    #[test]
    fn test_entry_path() {
        let f = Entry::new("Cargo.toml");

        assert_eq!(PathBuf::from("Cargo.toml"), f.path());
    }
}

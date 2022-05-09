//! Entry is for storing a file type (based on its extension) along with the pathname.
//!
//! That way we can have a different function to manage Gzip archives, Zip ones, etc.
//!

// Std library
//
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::PathBuf;

// Our crates
//
use crate::filetype::{ext_to_ftype, Input};

use anyhow::{anyhow, Result};

/// Entry carries the file path and its type (Plain, Gzip, etc.).
///
#[derive(Debug, Ord, PartialOrd, Eq, PartialEq)]
pub struct Entry {
    /// Pathname
    pub p: PathBuf,
    /// File type as found by `ext_to_ftype()`
    pub ft: Input,
}

impl Default for Entry {
    fn default() -> Self {
        Entry::new(&PathBuf::from(""))
    }
}

impl Entry {
    /// Create a new `Entry`  with the file type
    ///
    /// Example:
    /// ```
    /// use std::path::PathBuf;
    /// use dmarc_rs::entry::Entry;
    /// use dmarc_rs::filetype::Input;
    ///
    /// let f = Entry::new(&PathBuf::from("Foo.zip"));
    ///
    /// println!("{:?}", f.ft);
    /// ```
    ///
    pub fn new(p: &PathBuf) -> Self {
        Entry {
            p: p.to_owned(),
            ft: ext_to_ftype(&p),
        }
    }

    /// Allow for changing the file type
    ///
    /// Example:
    /// ```
    /// use dmarc_rs::entry::Entry;
    /// use dmarc_rs::filetype::Input;
    ///
    /// // This is obviously wrong, don't do it :)
    /// let f = Entry::from("Foo.zip").set(Input::Gzip);
    ///
    /// println!("{:?}", f.ft);
    /// ```
    ///
    pub fn set(mut self, t: Input) -> Self {
        self.ft = t;
        self
    }

    /// Open the given file and return the content as a String.
    ///
    /// This is where we call the different functions for the different types of
    /// input files.
    ///
    /// NOTE: plain files are assumed to be XML.
    ///
    pub fn get_data(self) -> Result<String> {
        match self.ft {
            Input::Csv|Input::Xml|Input::Plain => {
                let fh = match File::open(&self.p) {
                    Ok(fh) => fh,
                    Err(e) => return Err(anyhow!("{}", e.to_string())),
                };
                let mut lines = BufReader::new(fh);
                let mut s = String::new();
                let _cnt = lines.read_to_string(&mut s);
                Ok(s)
            },
            Input::Zip => unimplemented!(),
            Input::Gzip => unimplemented!(),
        }
    }
}

impl From<&str> for Entry {
    /// Convert a string slice into a PathBuf
    ///
    /// Example:
    /// ```
    /// use dmarc_rs::entry::Entry;
    /// let e = Entry::from("foo.zip");
    /// ```
    ///
    fn from(path: &str) -> Self {
        let p = PathBuf::from(path);
        Entry {
            p: p.to_owned(),
            ft: ext_to_ftype(&p),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case("", Input::Plain)]
    #[case("foo", Input::Plain)]
    #[case("foo.zip", Input::Zip)]
    #[case("bar.gz", Input::Gzip)]
    #[case("baz.xml.gz", Input::Gzip)]
    fn test_new(#[case] p: &str, #[case] res: Input) {
        let p = PathBuf::from(p);
        let e = Entry::new(&p);
        assert_eq!(res, e.ft);
    }

    #[rstest]
    #[case("", Input::Plain)]
    #[case("foo", Input::Plain)]
    #[case("foo.zip", Input::Zip)]
    #[case("bar.gz", Input::Gzip)]
    #[case("baz.xml.gz", Input::Gzip)]
    fn test_from(#[case] p: &str, #[case] res: Input) {
        let e = Entry::from(p);
        let f = Entry::new(&PathBuf::from(p));
        assert_eq!(res, e.ft);
        assert_eq!(f, e);
    }

    #[test]
    fn test_set() {
        let e = Entry::from("foo").set(Input::Gzip);
        assert_eq!(Input::Gzip, e.ft);
    }

    #[test]
    fn test_entry_get_xml() {
        let f = Entry::from("Cargo.toml");

        let txt = f.get_data();
        assert!(txt.is_ok());
        let txt = txt.unwrap();
        assert!(txt.contains("dmarc-rs"))
    }
}

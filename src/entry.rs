//! Entry is for storing a file type (based on its extension) along with the pathname.
//!
//! That way we can have a different function to manage Gzip archives, Zip ones, etc.
//!

// Std library
//
use std::fmt::{Debug, Display, Formatter};
use std::fs;
use std::path::PathBuf;

use eyre::{eyre, Result};
use tracing::trace;

// Our crates
//
use crate::filetype::Input;

/// Entry carries the file path and its type (Plain, Gzip, etc.).
///
#[derive(Clone)]
pub struct Entry {
    /// Pathname if any, `<stdin>` otherwise
    p: PathBuf,
    /// File type as found by `Input::from_path(&str)` or through `-t`
    ft: Input,
}

impl Default for Entry {
    fn default() -> Self {
        Self {
            p: PathBuf::new(),
            ft: Input::Unknown,
        }
    }
}

impl Display for Entry {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.p.to_string_lossy())
    }
}

impl Debug for Entry {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Entry::File")
            .field("p", &self.p)
            .field("ft", &self.ft)
            .finish()
    }
}

impl Entry {
    /// Create a new `Entry`  with the file type
    ///
    /// Example:
    /// ```
    /// use dmarc_rs::Entry;
    ///
    /// let f = Entry::from_str("Foo.zip");
    ///
    /// println!("{:?}", f.input_type());
    /// ```
    ///
    #[tracing::instrument]
    pub fn from_str(value: &str) -> Self {
        let ft = Input::from(value);
        Entry {
            p: PathBuf::from(value),
            ft,
        }
    }

    /// Return the stored path
    ///
    #[inline]
    pub fn path(&self) -> Result<PathBuf> {
        Ok(self.p.clone())
    }

    /// Return the Input type of the concerned entry
    ///
    #[inline]
    pub fn input_type(&self) -> Input {
        self.ft.clone()
    }

    pub fn with(&mut self, input: Input) -> &mut Self {
        self.ft = input;
        self
    }

    /// Open the given file and return the content as a String.
    ///
    /// This is where we call the different functions for the different types of
    /// input files.
    ///
    /// **NOTE** plain files are assumed to be XML.
    ///
    /// Example:
    /// ```
    /// # use eyre::eyre;
    /// use dmarc_rs::Entry;
    ///
    /// let f = Entry::from_str("foo.xml");
    ///
    /// let xml = match f.fetch() {
    ///     Ok(s) => s,
    ///     Err(e) => eyre!("Error reading.").to_string(),
    /// };
    /// ```
    ///
    #[tracing::instrument(skip(self))]
    pub fn fetch(&self) -> Result<String> {
        let res = fs::read_to_string(&self.p)?;
        trace!("read {} bytes", res.len());

        // We have the raw, possibly compressed in `res`
        //
        // Now see the file content
        //
        let s = match &self.input_type() {
            Input::Csv | Input::Xml => res,
            Input::Zip => "unimplemented".to_string(),
            Input::Gzip => "unimplemented".to_string(),
            Input::TarGzip => "unimplemented".to_string(),
            Input::Unknown => return Err(eyre!("invalid file content")),
        };
        Ok(s)
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case("", Input::Unknown)]
    #[case("foo", Input::Unknown)]
    #[case("foo.zip", Input::Zip)]
    #[case("bar.gz", Input::Gzip)]
    #[case("baz.xml.gz", Input::Gzip)]
    fn test_new(#[case] p: &str, #[case] res: Input) {
        let e = Entry::from_str(p);
        assert_eq!(res, e.ft);
    }

    #[rstest]
    #[case("", Input::Unknown)]
    #[case("foo", Input::Unknown)]
    #[case("foo.zip", Input::Zip)]
    #[case("bar.gz", Input::Gzip)]
    #[case("baz.xml.gz", Input::Gzip)]
    fn test_from(#[case] p: &str, #[case] res: Input) {
        let f = Entry::from_str(p);
        assert_eq!(res, f.ft);
    }

    #[test]
    fn test_set() {
        let mut e = Entry::from_str("foo");
        e.with(Input::Gzip);
        assert_eq!(Input::Gzip, e.ft);
    }

    #[test]
    fn test_entry_get_data() {
        let f = Entry::from_str("Cargo.toml");

        let txt = f.fetch();
        assert!(txt.is_err());
    }
}

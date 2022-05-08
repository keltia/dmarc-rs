//! Entry is for storing a file type (based on its extension) along with the pathname.
//!
//! That way we can have a different function to manage Gzip archives, Zip ones, etc.
//!

// Std library
//
use std::fs::File;
use std::path::PathBuf;

// Our crates
//
use crate::filetype::{ext_to_ftype, Input};

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

    /// Return a handle to the opened file.  This where the file type make all the differences
    /// and allow to manage archives.
    ///
    /// Example:
    /// ```no_run
    /// use dmarc_rs::entry::Entry;
    ///
    /// let fh = match Entry::from("Foo.zip").open() {
    ///     Ok(fh) => fh,
    ///     Err(e) => panic!("Error: {}", e.to_string()),
    /// };
    /// ```
    ///
    pub fn open(&self) -> std::io::Result<File> {
        match self.ft {
            Input::Plain => File::open(&self.p),
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

    #[test]
    fn test_set() {
        let e = Entry::new(&PathBuf::from("foo")).set(Input::Gzip);
        assert_eq!(Input::Gzip, e.ft);
    }
}

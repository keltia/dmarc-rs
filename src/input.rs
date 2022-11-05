//! Input file type handling
//!
//! We allow different type of files as input to `dmarc-cat`:
//!
//! - `csv` for CSV files instead of XML
//! - `xml` for plain XML
//! - `gzip` for gzipped XML
//! - `zip` for Zip files often containing both CSV and XML versions
//!
//! We also accept the following aliases:
//!
//! - `txt` for plain XML
//! - `gz` for gzip files.
//!
//! The name is not case-sensitive as seen in the tests below.
//!

// Std Library
//
use std::path::PathBuf;

// External crates
//

/// Allowed type of input, default for plain text is XML
///
#[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub enum Input {
    /// Plain CSV files
    Csv,
    /// XML compressed with gzip
    Gzip,
    /// Actual XML files
    Xml,
    /// ZIP files with generally both CSV and XML
    Zip,
    /// Nothing aka invalid
    None,
}

impl Input {
    /// Return a guess of the Input based on the extension otherwise return None
    ///
    /// ```rust
    /// # use dmarc_rs::input::Input;
    ///
    /// let ft = Input::from_path("foo.csv");
    /// assert_eq!(Input::Csv, ft);
    /// ```
    ///
    pub fn from_path<T: Into<PathBuf>>(p: T) -> Self {
        let pp = p.into();
        Input::from(
            pp.extension()
                .unwrap()
                .to_ascii_lowercase()
                .to_str()
                .unwrap(),
        )
    }
}

impl From<&str> for Input {
    /// Basic check from an &str like checking a CLI flag
    ///
    /// Can it be used to check a file's extension (see above `from_path()`)
    ///
    fn from(s: &str) -> Self {
        match s {
            "csv" => Input::Csv,
            "txt" => Input::Xml,
            "gzip" => Input::Gzip,
            "gz" => Input::Gzip,
            "xml" => Input::Xml,
            "zip" => Input::Zip,
            _ => Input::None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;
    use std::path::PathBuf;

    #[rstest]
    #[case("foo", Input::Xml)]
    #[case("foo.txt", Input::Xml)]
    #[case("foo.zip", Input::Zip)]
    #[case("foo.ZIP", Input::Zip)]
    #[case("foo.gz", Input::Gzip)]
    #[case("foo.GZ", Input::Gzip)]
    #[case("foo.Gz", Input::Gzip)]
    #[case("foo.xml", Input::Xml)]
    #[case("foo.XML", Input::Xml)]
    #[case("foo.csv", Input::Csv)]
    #[case("foo.CSV", Input::Csv)]
    #[case(".CSV", Input::None)]
    fn test_input_from_path(#[case] f: &str, #[case] t: Input) {
        assert_eq!(t, Input::from_path(p));
    }
}

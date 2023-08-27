//! Input file type handling
//!
//! We allow different type of files as input to `dmarc-cat`:
//!
//! - `plain` for plain XML
//! - `gzip` for gzipped XML
//! - `zip` for Zip files containing both CSV and XML versions
//!
//! We also accept the following aliases:
//!
//! - `txt` for plain files
//! - `gz` for gzip files.
//!
//! The name is not case-sensitive as seen in the tests below.
//!
//! `valid_input()` returns one of the `Input` enum values or an error.
//!

// Std Library
//
use std::ffi::OsStr;
use std::fmt::Debug;
use std::path::PathBuf;

// External crates
//
use clap::ValueEnum;

/// Allowed type of input
///
#[derive(
    Clone, Copy, Debug, Default, Ord, PartialOrd, Eq, PartialEq, strum::Display, ValueEnum,
)]
#[strum(serialize_all = "lowercase")]
pub enum Input {
    /// Plain CSV files
    Csv,
    /// XML compressed with gzip
    Gzip,
    /// Actual XML files
    /// plain text files are assumed to be XML
    #[default]
    Xml,
    /// ZIP files with generally both CSV and XML
    Zip,
    /// Tar+Gz
    TarGzip,
    /// Error type
    Unknown,
}

impl<T> From<T> for Input
where
    T: Into<PathBuf> + Debug + Clone,
{
    /// Matches a filename to a given input type based on the extension.
    /// Assumes stdin/- is plain text unless specified elsewhere
    ///
    #[tracing::instrument]
    fn from(value: T) -> Self {
        // all lowercase
        //
        let p: PathBuf = value.clone().into();
        let p1 = value.clone().into().to_string_lossy().to_ascii_lowercase();

        let ext = match p.extension() {
            Some(ext) => ext,
            _ => OsStr::new("txt"),
        }
        .to_string_lossy()
        .to_ascii_lowercase();

        if ext == "gz" {
            // Check for `.tar.gz`
            if p1.ends_with(".tar.gz") || p1.ends_with(".tgz") {
                return Input::TarGzip;
            }
        }
        // Match the rest
        //
        match ext.as_ref() {
            "csv" => Input::Csv,
            "zip" => Input::Zip,
            "txt" => Input::Xml,
            "xml" => Input::Xml,
            "tgz" => Input::TarGzip,
            "gz" => Input::Gzip,
            _ => Input::Unknown,
        }
    }
}

impl Input {
    pub fn valid(self) -> bool {
        self != Input::Unknown
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

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
    #[case("foo.tar.GZ", Input::TarGzip)]
    #[case("foo.tGZ", Input::TarGzip)]
    #[case("foo.bar", Input::Unknown)]
    fn test_input_from_str(#[case] f: &str, #[case] t: Input) {
        assert_eq!(t, Input::from(f))
    }

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
    #[case(".CSV", Input::Unknown)]
    #[case("foobar", Input::Unknown)]
    #[case("qZip", Input::Unknown)]
    #[case("", Input::Unknown)]
    fn test_valid_input(#[case] s: &str, #[case] it: Input) {
        let r = Input::from(s);
        assert!(r.valid());
    }
}

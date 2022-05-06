//! File handling
//!

// Std library
//
use std::ffi::OsStr;
use std::fs::File;
use std::io;
use std::path::{Path, PathBuf};

// Internal crates
//
use crate::analyze::analyze_file;
use dmarc_rs::filetype::*;

// External crates
//
use anyhow::{anyhow, Result};

pub fn handle_stream(fin: &dyn io::BufRead, ftype: Input) {}

/// Matches a filename to a given input type based on the extension.
/// Assumes stdin/- is plain text unless specified elsewere
///
pub fn ext_to_ftype(p: &Path) -> Input {
    let ext = match p.extension() {
        Some(ext) => ext,
        _ => OsStr::new("txt"),
    }
    .to_owned();
    match ext.into_string().unwrap().to_lowercase().as_str() {
        "zip" => Input::Zip,
        "txt" => Input::Plain,
        "xml" => Input::Plain,
        "gz" => Input::Gzip,
        _ => Input::Plain,
    }
}

/// Check if every file in the list and only return the list of valid ones.
///
pub fn check_for_files(lfn: &Vec<PathBuf>) -> Vec<PathBuf> {
    let mut res: Vec<PathBuf> = vec![];

    // Check for various files.
    //
    for f in lfn.iter() {
        if f.exists() || *f == PathBuf::from("-") {
            res.push(f.to_owned());
            println!("file: {:?}", f);
        } else {
            println!("Unknown file {:?}", f);
            continue;
        }
    }
    res
}

/// Scan the list of files and run `analyze_file()`  on each of them
/// accumulating results.
///
pub fn scan_list(lfn: &Vec<PathBuf>) -> Result<String> {
    let mut r = vec![];
    let mut failed = vec![];

    for fp in lfn {
        let mut fh = match File::open(fp) {
            Ok(fh) => fh,
            Err(e) => {
                log::warn!("Warning: can't open {:?}: {}", fp, e.to_string());
                failed.push(fp.to_str().unwrap());
                continue;
            }
        };
        r.push(analyze_file(&mut fh).unwrap());
    }
    if failed.is_empty() {
        return Ok(r.join("/"));
    }
    return Err(anyhow!("{:?}", failed));
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case("foo.zip", Input::Zip)]
    #[case("foo.ZIP", Input::Zip)]
    #[case("foo.gz", Input::Gzip)]
    #[case("foo.GZ", Input::Gzip)]
    #[case("foo.Gz", Input::Gzip)]
    #[case("foo.xml", Input::Plain)]
    #[case("foo.XML", Input::Plain)]
    #[case("foo.csv", Input::Plain)]
    #[case("foo.CSV", Input::Plain)]
    #[case(".CSV", Input::Plain)]
    fn test_ext_to_ftype(#[case] f: PathBuf, #[case] t: Input) {
        let p = PathBuf::from(f);
        assert_eq!(t, ext_to_ftype(&p))
    }

    #[test]
    fn test_scan_list_empty() {
        let r = scan_list(&vec![PathBuf::from("")]);
        assert!(r.is_err())
    }

    #[test]
    fn test_scan_list_nonexistent() {
        let r = scan_list(&vec![PathBuf::from("/nonexistent")]);
        assert!(r.is_err())
    }

    #[test]
    fn test_check_for_unknown_files() {
        let l = vec![
            PathBuf::from("foo"),
            PathBuf::from("bar")
        ];

        let l2 = check_for_files(&l);
        assert!(l2.is_empty())
    }

    #[test]
    fn test_check_for_partial_files() {
        let l = vec![
            PathBuf::from("foo"),
            PathBuf::from("bar"),
            PathBuf::from("Cargo.toml"),
        ];

        let l2 = check_for_files(&l);
        assert!(!l2.is_empty());
        assert_eq!(vec![PathBuf::from("Cargo.toml")], l2);
    }

    #[test]
    fn test_check_for_partial_files_stdin() {
        let l = vec![
            PathBuf::from("foo"),
            PathBuf::from("bar"),
            PathBuf::from("-"),
            PathBuf::from("Cargo.toml"),
        ];

        let l2 = check_for_files(&l);
        assert_eq!(2, l2.len());
        assert_eq!(vec![PathBuf::from("-"), PathBuf::from("Cargo.toml")], l2);
    }

}
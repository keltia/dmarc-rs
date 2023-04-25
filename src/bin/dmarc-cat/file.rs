//! File handling
//!

// Std library
//
use std::fs::File;
use std::io;
use std::path::PathBuf;

// Internal crates
//
use crate::analyze::analyze_file;
use dmarc_rs::entry::Entry;
use dmarc_rs::filetype::*;

// External crates
//
use anyhow::{anyhow, Result};
use log::{info, trace, warn};

/// Check if every file in the list and only return the list of valid ones.
///
macro_rules! getpaths {
    ($file:ident) => {
        $file.iter().map(|f| f.path()).collect::<Vec<PathBuf>>()
    };
}

/// Scan the list of files and run `handle_one_file()`  on each of them
/// accumulating results.
///
pub fn scan_list(lfn: &mut Vec<Entry>) -> Result<String> {
    // loop over each entry, resolution is done in parallel through rayon, no need to do the
    // the same here
    //
    // sorting out all which succeeded and those which failed
    //
    let mut res: Vec<String> = vec![];

    let (rr, failed): (Vec<_>, Vec<_>) = lfn
        .iter_mut()
        .inspect(|f| info!("looking at {:?}", f))
        .partition(|&f| {
            let mut f = f.clone();
            if let Ok(s) = f.fetch() {
                res.push(s);
                true
            } else {
                false
            }
        });

    // List of succeeded entries
    dbg!(&rr);
    // List of failed ones
    dbg!(&failed);

    // collect all succeeded entries
    //
    let res = rr
        .iter()
        .fold("", |res, &f| (res.to_string() + &f.res).as_str());

    info!("succeeded files: {:?}", getpaths!(rr));

    // Get all failed path names
    //
    let errlist: Vec<PathBuf> = getpaths!(failed);
    info!("failed files: {:?}", errlist);

    Ok(res.to_string())
}

pub fn handle_one_file(e: &PathBuf) -> Result<String> {
    match e.ft {
        Input::Csv => Ok("csv".to_string()),
        Input::Plain => Ok("txt".to_string()),
        Input::Xml => Ok("xml".to_string()),
        Input::Zip => Ok("zip".to_string()),
        Input::Gzip => Ok("gzip".to_string()),
    };

    Ok("Nope".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scan_list_empty() {
        let r = scan_list(&vec![Entry {
            p: PathBuf::from(""),
            ft: Input::Plain,
        }]);
        assert!(r.is_err())
    }

    #[test]
    fn test_scan_list_nonexistent() {
        let r = scan_list(&vec![Entry {
            p: PathBuf::from("/nonexistent"),
            ft: Input::Plain,
        }]);
        assert!(r.is_err())
    }

    #[test]
    fn test_check_for_unknown_files() {
        let l = vec![PathBuf::from("foo"), PathBuf::from("bar")];

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
        assert_eq!(
            vec![Entry {
                p: PathBuf::from("Cargo.toml"),
                ft: Input::Plain
            }],
            l2
        );
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
        assert_eq!(
            vec![
                Entry {
                    p: PathBuf::from("-"),
                    ft: Input::Plain
                },
                Entry {
                    p: PathBuf::from("Cargo.toml"),
                    ft: Input::Plain
                },
            ],
            l2
        );
    }
}

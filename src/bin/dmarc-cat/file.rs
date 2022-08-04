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

pub fn handle_stream(fin: &dyn io::BufRead, ftype: Input) {
    unimplemented!()
}

/// Check if every file in the list and only return the list of valid ones.
///
pub fn check_for_files(lfn: &[PathBuf]) -> Vec<Entry> {
    let mut res: Vec<Entry> = vec![];

    // Check for various files.
    //
    for f in lfn.iter() {
        if f.exists() || *f == PathBuf::from("-") {
            res.push(Entry::new(f));
            println!("file: {:?}", f);
        } else {
            println!("Unknown file {:?}", f);
            continue;
        }
    }
    res
}

use rayon::prelude::*;

/// Scan the list of files and run `handle_one_file()`  on each of them
/// accumulating results.
///
pub fn scan_list(lfn: &Vec<Entry>) -> Result<String> {
    let mut failed = vec![];

    // rr: raw results
    //
    let rr: Vec<Result<String>> = lfn
        .par_iter()
        .map(|f| match handle_one_file(&f) {
            Err(e) => {
                log::warn!("Warning: can't open {:?}: {}", &f.p, e.to_string());
                failed.push(&f.p)
            }
            _ => (),
        })
        .collect();

    if failed.is_empty() {
        return Ok(rr.join("/"));
    }
    Err(anyhow!("{:?}", failed))
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

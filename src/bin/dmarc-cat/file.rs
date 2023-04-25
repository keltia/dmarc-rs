//! File handling
//!

// Std library
//
use std::fs;
use std::path::PathBuf;

// Internal crates
//
use dmarc_rs::input::*;

// External crates
//
use anyhow::{anyhow, Result};
use dmarc_rs::entry::Entry;
use dmarc_rs::filetype::Input;
use log::info;

pub fn handle_stream(fin: &dyn io::BufRead, ftype: Input) {
    unimplemented!()
}

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

pub fn handle_one_file(e: &Entry) -> Result<String> {
    match e.ft {
        Input::Csv => Ok("csv".to_string()),
        Input::Plain => Ok("txt".to_string()),
        Input::Xml => Ok("xml".to_string()),
        Input::Zip => Ok("zip".to_string()),
        Input::Gzip => Ok("gzip".to_string()),
        Input::Unknown => Err(anyhow!("bad file type")),
    };

    Ok("Nope".into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use dmarc_rs::makelist;

    #[test]
    fn test_scan_list_empty() {
        let r = scan_list(&mut vec![PathBuf::from("")]);
        dbg!(&r);
        assert!(r.is_err())
    }

    #[test]
    fn test_scan_list_nonexistent() {
        let r = scan_list(&mut vec![PathBuf::from("/nonexistent")]);
        dbg!(&r);
        assert!(r.is_err())
    }
}

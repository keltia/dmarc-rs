//! Module handling the DNS resolving operations
//!
//! We use `IpList` as container and define the internal functions called by `IpList::simple_solve()`
//! and `IpList::parallel_solve()`  to return the same list with all names hopefully resolved.
//!
//! BUGS: this version only handle one name per IP (whatever is returned by `lookup_addr()`.
//!

// Our crates
//
use crate::ip::Ip;
use crate::iplist::IpList;

// Std library
//
use std::error::Error;
use std::sync::mpsc::{channel, Receiver};
use std::thread;

// External crates
//
use anyhow::{anyhow, Result};
use threadpool::ThreadPool;

/// `resolve()` is the main function call to get all names from the list of `Ip` we get from the
/// XML file.
///
///
pub fn resolve(ipl: &IpList, njobs: usize) -> Result<IpList> {
    let max_threads = num_cpus::get();

    // Put a hard limit on how many parallel thread to the max number of cores (incl. Hyperthreading).
    //
    if njobs > max_threads {
        return Err(anyhow!("Too many threads"))
    }

    // Call the appropriate one
    //
    match njobs {
        1 => Ok(ipl.simple_solve()),
        _ => Ok(ipl.parallel_solve(njobs)),
    }
}

/// Start enough workers to resolve IP into PTR.
///
pub(crate) fn fan_out(
    rx_gen: Receiver<Ip>,
    pool: ThreadPool,
    njobs: usize,
) -> Result<Receiver<Ip>, Box<dyn Error>> {
    let (tx, rx) = channel();

    for _ in 0..njobs {
        let tx = tx.clone();
        let n = rx_gen.recv().unwrap();
        pool.execute(move || {
            let r = n.solve();
            tx.send(r).expect("waiting channel");
        });
    }
    Ok(rx)
}

/// Gather all results into an output channel
///
pub(crate) fn fan_in(rx_out: Receiver<Ip>) -> Result<Receiver<Ip>, Box<dyn Error>> {
    let (tx, rx) = channel();
    thread::spawn(move || {
        for ip in rx_out.iter() {
            tx.send(ip).expect("can not send");
        }
    });
    Ok(rx)
}

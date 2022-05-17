//! Module handling the DNS resolving operations
//!
//! We use `IpList` as container and `resolve()` is the main function to get all names.  As we have
//! the choice between two solvers, you can select the simple single-threaded one by specifying
//! that you want only 1 job.
//!
//! When the crate is compiled, the number of CPU & CPU threads is read and that gives us the upper
//! bound for the parallelism.  The `dmarc-cat` binary will default to number physical cores but the hard
//! limit is the number of total core threads (which is higher if the CPU supports Hyperthreading).
//!
//! **NOTE** I have no idea how CPU with different cores types (Apple M1 family or others) are handled,
//! not sure it would make any difference in this case.
//!
//! **BUGS** this version only handle one name per IP (whatever is returned by `lookup_addr()`.
//!

use std::fmt::{Debug, Formatter};
use std::io::Error;
use std::net::IpAddr;
// Our crates
//
use crate::iplist::IpList;

// Std library
//

// External crates
//
use anyhow::{anyhow, Result};
use crate::resolve;

/// This trait will allow us to override the resolving function during tests.
pub trait Resolver {
    /// Get the PTR record associated with `ip`.
    fn lookup_addr(ip: &IpAddr) -> Result<String, std::io::Error>;
}

pub struct RealSolver{}

impl Resolver for RealSolver {
    fn lookup_addr(ip: &IpAddr) -> Result<String, Error> {
        dns_lookup::lookup_addr(ip)
    }
}

impl Debug for RealSolver {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

pub struct DumbSolver {}

impl Resolver for DumbSolver {
    fn lookup_addr(ip: &IpAddr) -> Result<String, Error> {
        Ok("dumb.host.name".into())
    }
}

impl Debug for DumbSolver {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

/// `resolve()` is the main function call to get all names from the list of `Ip` we get from the
/// XML file.
///
/// Example:
/// ```no_run
/// # use dmarc_rs::resolve::resolve;
/// # use dmarc_rs::iplist::IpList;
/// let l = IpList::from(["1.1.1.1", "2606:4700:4700::1111", "192.0.2.1"]);
///
/// // Using the simple single threaded solver.
/// let ptr = resolve(&l, 1).unwrap();
///
/// // Use the parallel solver with as many threads as the CPU has.
/// let ptr2 = resolve(&l, num_cpus::get()).unwrap();
/// ```
///
pub fn resolve<R: resolve::Resolver>(ipl: &IpList, njobs: usize) -> Result<IpList> {
    let max_threads = num_cpus::get();

    // Put a hard limit on how many parallel thread to the max number of cores (incl. Hyperthreading).
    //
    if njobs > max_threads {
        return Err(anyhow!("Too many threads"));
    }

    // Call the appropriate one
    //
    match njobs {
        1 => Ok(ipl.simple_solve()),
        _ => Ok(ipl.parallel_solve(njobs)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invalid_jobs() {
        let l = IpList::from(["1.1.1.1", "2606:4700:4700::1111", "192.0.2.1"]);
        assert!(resolve(&l, 1000).is_err())
    }

    #[test]
    fn test_resolve() {
        let l = IpList::from(["1.1.1.1", "2606:4700:4700::1111", "192.0.2.1"]);

        // Using the simple single threaded solver.
        let ptr = resolve(&l, 1).unwrap();

        // Use the parallel solver with 4 threads.
        let ptr2 = resolve(&l, num_cpus::get()).unwrap();

        assert_eq!(ptr, ptr2);
    }
}

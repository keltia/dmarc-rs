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
//! We define a list of IP tuples from the `dmarc_rs::ip` crate and implement two methods
//! for resolving the IP into names.  One is `simple_solve()` which is a straightforward sequential
//! solver, the other one is `parallel_solve()` which is using threads from a pool to implement a
//! worker-based fan-out/fan-in scheme with channels to move data around.
//!
//! You can select the resolving module to be used from the three defined in `dmarc_rs::resolver`.
//!
//! Examples:
//! ```rust
//! # use dmarc_rs::iplist::IpList;
//! # use dmarc_rs::resolver::*;
//! let l = IpList::from(["1.1.1.1", "2606:4700:4700::1111", "192.0.2.1"]);
//! let res = res_init(ResType::Real);
//!
//! // Use the simple solver
//! let ptr = resolve(l, 1, &res);
//! dbg!(&ptr);
//! ```
//! and with the parallel solver but with the default resolver:
//! ```rust
//! # use dmarc_rs::iplist::IpList;
//! # use dmarc_rs::resolver::*;
//! // Get the number of physical cores, I prefer to use this one instead of the potentially
//! // larger total cores because Hyperthreading has some overhead.
//! let njobs = num_cpus::get_physical();
//!
//! let l = IpList::from(["1.1.1.1", "2606:4700:4700::1111", "192.0.2.1"]);
//! let res = res_init(ResType::default());
//!
//! // Use the parallel solver
//! let ptr = parallel_solve(l, njobs, res);
//! dbg!(&ptr);
//! ```
//!

// Our crates
//
use dmarc_rs::ip::Ip;
use dmarc_rs::iplist::IpList;
use dmarc_rs::resolver::*;

// Std library
//
use std::sync::mpsc::{channel, Receiver};
use std::thread;

// External crates
//
use anyhow::{anyhow, Result};
use threadpool::ThreadPool;

/// `resolve()` is the main function call to get all names from the list of `Ip` we get from the
/// XML file.
///
/// Example:
/// ```no_run
/// # use dmarc_rs::iplist::IpList;
/// let l = IpList::from(["1.1.1.1", "2606:4700:4700::1111", "192.0.2.1"]);
///
/// // Select a resolver
/// let res = res_init(ResType::Real);
///
/// // Using the simple single threaded solver.
/// let ptr = resolve(l, 1, res).unwrap();
///
/// // Use the parallel solver with as many threads as the CPU has.
/// let ptr2 = resolve(l, num_cpus::get(), res).unwrap();
/// ```
///
pub fn resolve(ipl: IpList, njobs: usize, res: &Solver) -> Result<IpList> {
    let max_threads = num_cpus::get();

    // Put a hard limit on how many parallel thread to the max number of cores (incl.
    // avoid overhead even on modern versions of Hyperthreading).
    //
    if njobs > max_threads {
        return Err(anyhow!("Too many threads"));
    }

    // Call the appropriate one
    //
    match njobs {
        1 => Ok(simple_solve(&ipl, res)),
        _ => Ok(parallel_solve(ipl, njobs, res)),
    }
}

/// Simple and straightforward sequential solver
///
/// Example:
/// ```
/// # use dmarc_rs::iplist::IpList;
/// # use dmarc_rs::ip::Ip;
/// let l = IpList::from(["1.1.1.1", "2606:4700:4700::1111", "192.0.2.1"]);
///
/// // select a given resolver
/// let res = RealResolver{};
///
/// let ptr = res.simple_solve(l);
/// ```
///
fn simple_solve(ipl: &IpList, res: &Solver) -> IpList {
    let mut r: IpList = ipl
        .clone()
        .into_iter()
        .map(|ip| res.solve(&ip))
        .collect();
    r.sort();
    r
}

/// Convert a list of IP into names with multiple threads
///
/// It uses a function to fill in the input channel then fan_out/fan_in to fill the result
/// list.
///
/// Example:
/// ```no_run
/// # use dmarc_rs::ip::Ip;
/// # use dmarc_rs::iplist::IpList;
/// let l = IpList::from(["1.1.1.1", "2606:4700:4700::1111", "192.0.2.1"]);
///
/// // Select a resolver
/// let r = RealResolver{};
///
/// let ptr = parallel_solve(l, 4, r);
/// ```
///
fn parallel_solve(ipl: IpList, njobs: usize, res: &Solver) -> IpList {
    let mut full = IpList::new();
    let s = ipl.len();

    let pool = ThreadPool::new(njobs);
    let rx_gen = queue(ipl).unwrap();
    let rx_out = fan_out(rx_gen, pool, s, res).unwrap();
    for ip in fan_in(rx_out).unwrap() {
        full.push(ip);
    }
    full.sort();
    dbg!(&full);
    full
}

/// Take all values from the list and send them into a queue+
///
fn queue(ipl: IpList) -> Result<Receiver<Ip>> {
    let (tx, rx) = channel();

    // construct a copy of the list
    //let all: Vec<Ip> = ipl.l.clone();

    // use that copy to send over
    thread::spawn(move || {
        for ip in ipl.into_iter() {
            tx.send(ip.clone()).expect("can not queue")
        }
    });
    Ok(rx)
}

/// Start enough workers to resolve IP into PTR.
///
fn fan_out(
    rx_gen: Receiver<Ip>,
    pool: ThreadPool,
    njobs: usize,
    res: &Solver,
) -> Result<Receiver<Ip>, Box<dyn std::error::Error>> {
    let (tx, rx) = channel();

    for _ in 0..njobs {
        let tx = tx.clone();
        let n = rx_gen.recv().unwrap();

        let res = res.clone();
        pool.execute(move || {
            let name: Ip = res.solve(&n);
            tx.send(Ip {
                ip: name.ip,
                name: name.name,
            })
                .expect("waiting channel");
        });
    }
    Ok(rx)
}

/// Gather all results into an output channel
///
fn fan_in(rx_out: Receiver<Ip>) -> Result<Receiver<Ip>, Box<dyn std::error::Error>> {
    let (tx, rx) = channel();
    thread::spawn(move || {
        for ip in rx_out.iter() {
            tx.send(ip).expect("can not send");
        }
    });
    Ok(rx)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invalid_jobs() {
        let l = IpList::from(["1.1.1.1", "2606:4700:4700::1111", "192.0.2.1"]);
        let res = res_init(ResType::Fake);

        assert!(resolve(l, 1000, &res).is_err())
    }

    #[test]
    fn test_resolve() {
        let l = IpList::from(["1.1.1.1", "2606:4700:4700::1111", "192.0.2.1"]);
        let res = res_init(ResType::Fake);

        // Using the simple single threaded solver.
        let ptr = resolve(l, 1, &res).unwrap();

        // Use the parallel solver with 4 threads.
        let ptr2 = resolve(l, num_cpus::get(), &res).unwrap();

        assert_eq!(ptr, ptr2);
    }

    #[test]
    fn test_parallel_solve_empty() {
        let a = IpList::new();
        let res = res_init(ResType::Fake);

        let r = parallel_solve(a, num_cpus::get_physical(), &res);

        assert!(r.is_empty())
    }

    #[test]
    fn test_simple_solve_empty() {
        let a = IpList::new();
        let res = res_init(ResType::Fake);

        let r = simple_solve(&a, &res);

        assert!(r.is_empty())
    }

    #[test]
    fn test_solve_empty() {
        let a = IpList::new();
        let res = res_init(ResType::Fake);

        let r = resolve(a, 1, &res);

        assert!(r.is_err());
        assert!(r.unwrap().is_empty());
    }

    #[test]
    fn test_dumb_simple_solve_ok() {
        let l = IpList::from(["1.1.1.1", "2606:4700:4700::1111", "192.0.2.1"]);
        let res = res_init(ResType::Fake);

        let ptr = simple_solve(&l, &res);

        assert_eq!(l.len(), ptr.len());
        assert_eq!("one.one.one.one", ptr[0].name.to_string());
        assert_eq!("some.host.invalid", ptr[1].name.to_string());
        assert_eq!("one.one.one.one", ptr[2].name.to_string());
    }

    #[test]
    fn test_dumb_parallel_solve_ok() {
        let l = IpList::from(["1.1.1.1", "2606:4700:4700::1111", "192.0.2.1"]);
        let res = res_init(ResType::Fake);

        let ptr = parallel_solve(l, num_cpus::get_physical(), &res);

        assert_eq!(l.len(), ptr.len());
        // Order is not always preserved so check inside
        //
        for x in ptr {
            if x.ip.to_string() == "192.0.2.1" {
                assert_eq!("some.host.invalid", x.name);
            } else {
                assert_eq!("one.one.one.one", x.name);
            }
        }
    }

    #[test]
    fn test_dumbsolver_solve() {
        let l = IpList::from(["1.1.1.1", "2606:4700:4700::1111", "192.0.2.1"]);
        let res = res_init(ResType::Fake);

        let r = resolve(l, 1, &res);
        assert!(r.is_ok());
        assert_eq!(l, r.unwrap());
    }
}

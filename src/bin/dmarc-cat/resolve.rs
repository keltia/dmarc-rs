//! Module handling the DNS resolving operations (SYNC)
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

/// Simple and straightforward sequential solver
///
/// Example:
/// ```
/// # use dmarc_rs::iplist::IpList;
/// # use dmarc_rs::ip::Ip;
/// let l = IpList::from(["1.1.1.1", "2606:4700:4700::1111", "192.0.2.1"]);
///
/// // select a given resolver
/// let res = res_init(ResType::Real);
///
/// let ptr = simple_solve(l, res);
/// ```
///
pub fn simple_solve(ipl: &IpList, res: &Solver) -> IpList {
    let mut r: IpList = ipl.clone().into_iter().map(|ip| res.solve(&ip)).collect();
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
/// let res = res_init(ResType::Real);
///
/// let ptr = parallel_solve(l, 4, res);
/// ```
///
pub fn parallel_solve(ipl: &IpList, njobs: usize, res: &Solver) -> IpList {
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

/// Take all values from the list and send them into a queue
///
fn queue(ipl: &IpList) -> Result<Receiver<Ip>> {
    let (tx, rx) = channel();

    // construct a copy of the list
    let all = ipl.clone();

    // use that copy to send over
    thread::spawn(move || {
        for ip in all.into_iter() {
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
    use crate::resolve;
    use super::*;

    #[test]
    fn test_invalid_jobs() {
        let l = IpList::from(["1.1.1.1", "2606:4700:4700::1111", "192.0.2.1"]);
        let res = res_init(ResType::Fake);

        assert!(resolve(&l, 1000, &res).is_err())
    }

    #[test]
    fn test_resolve() {
        let l = IpList::from(["1.1.1.1", "2606:4700:4700::1111", "192.0.2.1"]);
        let res = res_init(ResType::Fake);

        // Using the simple single threaded solver.
        let ptr = resolve(&l, 1, &res).unwrap();

        // Use the parallel solver with 4 threads.
        let ptr2 = resolve(&l, num_cpus::get(), &res).unwrap();

        assert_eq!(ptr, ptr2);
    }

    #[test]
    fn test_parallel_solve_empty() {
        let a = IpList::new();
        let res = res_init(ResType::Fake);

        let r = parallel_solve(&a, num_cpus::get_physical(), &res);

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
    #[should_panic]
    fn test_solve_empty() {
        let a = IpList::new();
        let res = res_init(ResType::Fake);

        let r = resolve(&a, 1, &res);

        assert!(r.is_err());
        assert!(r.unwrap().is_empty());
    }

    #[test]
    fn test_dumb_simple_solve_ok() {
        let l = IpList::from(["1.1.1.1", "2606:4700:4700::1111", "192.0.2.1"]);
        let res = res_init(ResType::Fake);

        let ptr = simple_solve(&l, &res);

        assert_eq!(l.len(), ptr.len());
        for x in ptr {
            assert_eq!("some.host.invalid", x.name);
        }
    }

    #[test]
    fn test_dumb_parallel_solve_ok() {
        let l = IpList::from(["1.1.1.1", "2606:4700:4700::1111", "192.0.2.1"]);
        let res = res_init(ResType::Fake);

        let ptr = parallel_solve(&l, num_cpus::get_physical(), &res);

        assert_eq!(l.len(), ptr.len());
        // Order is not always preserved so check inside
        //
        for x in ptr {
            assert_eq!("some.host.invalid", x.name);
        }
    }

    #[test]
    fn test_fakesolver_resolve() {
        let l = IpList::from(["1.1.1.1", "2606:4700:4700::1111", "192.0.2.1"]);
        let res = res_init(ResType::Fake);

        let ptr = resolve(&l, 1, &res);
        assert!(ptr.is_ok());

        let ptr = ptr.unwrap();
        for x in ptr {
            assert_eq!("some.host.invalid", x.name);
        }
    }
}

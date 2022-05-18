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
//! Examples:
//! ```rust
//! # use dmarc_rs::iplist::IpList;
//! let l = IpList::from(["1.1.1.1", "2606:4700:4700::1111", "192.0.2.1"]);
//!
//! // Use the simple solver
//! let ptr = simple_solve();
//! dbg!(&ptr);
//! ```
//! and with the parallel solver:
//! ```rust
//! # use dmarc_rs::iplist::IpList;
//! // Get the number of physical cores, I prefer to use this one instead of the potentially
//! // larger total cores because Hyperthreading has some overhead.
//! let njobs = num_cpus::get_physical();
//! let l = IpList::from(["1.1.1.1", "2606:4700:4700::1111", "192.0.2.1"]);
//!
//! // Use the parallel solver
//! let ptr = parallel_solve(njobs);
//! dbg!(&ptr);
//! ```
//!
//! **BUGS** this version only handle one name per IP (whatever is returned by `lookup_addr()`.
//!

// Our crates
//
use dmarc_rs::ip::Ip;
use dmarc_rs::iplist::IpList;

// Std library
//
use std::fmt::{Debug, Formatter};
use std::io::Error;
use std::net::IpAddr;
use std::sync::mpsc::{channel, Receiver};
use std::thread;

// External crates
//
use anyhow::{anyhow, Result};
use threadpool::ThreadPool;

/// This trait will allow us to override the resolving function during tests.
///
pub trait Resolver {
    /// Get the PTR record associated with `ip`.
    fn lookup_addr(&self, ip: &IpAddr) -> Result<String, std::io::Error>;
    /// Get the IP 2 PTR for all elements in `IpList`
    fn solve(&self, ipl: IpList, njobs: usize) -> Result<IpList>;
}

/// Empty struct here only to be an implementation of the `Resolver` trait.
///
pub struct RealSolver {}

impl RealSolver {
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
    fn simple_solve(&self, ipl: &IpList) -> IpList {
        let mut r:IpList = ipl.clone().into_iter().map(|ip| {
            Ip {
                ip: ip.ip,
                name: self.lookup_addr(&ip.ip).unwrap(),
            }
        }).collect();
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
    fn parallel_solve(&self, ipl: IpList, njobs: usize) -> IpList {
        let mut full = IpList::new();
        let s = ipl.len();

        let pool = ThreadPool::new(njobs);
        let rx_gen = &self.queue(ipl).unwrap();
        let rx_out = self.fan_out(rx_gen, pool, s).unwrap();
        for ip in self.fan_in(rx_out).unwrap() {
            full.push(ip);
        }
        full.sort();
        dbg!(&full);
        full
    }

    /// Take all values from the list and send them into a queue+
    ///
    fn queue(&self, ipl: IpList) -> Result<Receiver<Ip>> {
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
        &self,
        rx_gen: &Receiver<Ip>,
        pool: ThreadPool,
        njobs: usize,
    ) -> Result<Receiver<Ip>, Box<dyn std::error::Error>> {
        let (tx, rx) = channel();

        for _ in 0..njobs {
            let tx = tx.clone();
            let n = rx_gen.recv().unwrap();

            pool.execute(move || {
                let name = &self.lookup_addr(&n.ip).unwrap();
                tx.send(Ip {
                    ip: n.ip,
                    name: name.to_owned(),
                })
                    .expect("waiting channel");
            });
        }
        Ok(rx)
    }

    /// Gather all results into an output channel
    ///
    fn fan_in(&self, rx_out: Receiver<Ip>) -> Result<Receiver<Ip>, Box<dyn std::error::Error>> {
        let (tx, rx) = channel();
        thread::spawn(move || {
            for ip in rx_out.iter() {
                tx.send(ip).expect("can not send");
            }
        });
        Ok(rx)
    }
}

impl Resolver for RealSolver {
    /// This is the function that does DNS resolution, from IP to name/PTR.
    ///
    /// We are currently using the `lookup_addr◌` implementation from the `dns_lookup` crate.
    ///
    fn lookup_addr(&self, ip: &IpAddr) -> Result<String, Error> {
        dns_lookup::lookup_addr(ip)
    }

    /// `solve()` is the main function call to get all names from the list of `Ip` we get from the
    /// XML file.
    ///
    /// Example:
    /// ```no_run
    /// # use dmarc_rs::iplist::IpList;
    /// let l = IpList::from(["1.1.1.1", "2606:4700:4700::1111", "192.0.2.1"]);
    ///
    /// // Select a resolver
    /// let res = RealResolver{};
    ///
    /// // Using the simple single threaded solver.
    /// let ptr = res.solve(&l, 1).unwrap();
    ///
    /// // Use the parallel solver with as many threads as the CPU has.
    /// let ptr2 = res.solve(&l, num_cpus::get()).unwrap();
    /// ```
    ///
    fn solve(&self, ipl: IpList, njobs: usize) -> Result<IpList> {
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
            1 => Ok(self.simple_solve(&ipl)),
            _ => Ok(self.parallel_solve(ipl, njobs)),
        }
    }
}

impl Debug for RealSolver {
    fn fmt(&self, _f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

/// Empty struct here only to be an implementation of the `Resolver` trait.
///
pub struct DumbSolver {}

impl Resolver for DumbSolver {
    /// Return an easy and totally wrong name.
    ///
    fn lookup_addr(&self, _ip: &IpAddr) -> Result<String, Error> {
        Ok("dumb.host.name".into())
    }

    /// Return the whole list unchanged.
    ///
    fn solve(&self, ipl: IpList, _njobs: usize) -> Result<IpList> {
        Ok(ipl)
    }
}

impl Debug for DumbSolver {
    fn fmt(&self, _f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invalid_jobs() {
        let l = IpList::from(["1.1.1.1", "2606:4700:4700::1111", "192.0.2.1"]);
        let res = Box::new(DumbSolver{});

        assert!(resolve(&l, 1000, res).is_err())
    }

    #[test]
    fn test_resolve() {
        let l = IpList::from(["1.1.1.1", "2606:4700:4700::1111", "192.0.2.1"]);
        let res = Box::new(DumbSolver{});

        // Using the simple single threaded solver.
        let ptr = resolve(&l, 1, res).unwrap();

        // Use the parallel solver with 4 threads.
        let ptr2 = resolve(&l, num_cpus::get(), res).unwrap();

        assert_eq!(ptr, ptr2);
    }

    #[test]
    fn test_parallel_solve_empty() {
        let a = IpList::new();
        let res = Box::new(DumbSolver{});

        let r = parallel_solve(&a, num_cpus::get_physical(), res);

        assert!(r.is_empty())
    }

    #[test]
    fn test_simple_solve_empty() {
        let a = IpList::new();
        let res = DumbSolver{};

        let r = res.simple_solve(&a);

        assert!(r.is_empty())
    }

    #[test]
    fn test_solve_empty() {
        let a = IpList::new();
        let res = DumbSolver{};

        let r = res.solve(a, 1);

        assert!(r.is_err());
        assert!(r.unwrap().is_empty());
    }

    #[test]
    fn test_dumb_simple_solve_ok() {
        let l = IpList::from(["1.1.1.1", "2606:4700:4700::1111", "192.0.2.1"]);
        let res = DumbSolver{};

        let ptr = res.simple_solve(&l);

        assert_eq!(l.len(), ptr.len());
        assert_eq!("one.one.one.one", ptr[0].name.to_string());
        assert_eq!("some.host.invalid", ptr[1].name.to_string());
        assert_eq!("one.one.one.one", ptr[2].name.to_string());
    }

    #[test]
    fn test_dumb_parallel_solve_ok() {
        let l = IpList::from(["1.1.1.1", "2606:4700:4700::1111", "192.0.2.1"]);
        let res = DumbSolver{};

        let ptr = res.parallel_solve(&l, num_cpus::get_physical());

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
        let res = DumbSolver{};

        let r = res.solve(l, 1);
        assert!(r.is_ok());
        assert_eq!(l, r.unwrap());
    }
}

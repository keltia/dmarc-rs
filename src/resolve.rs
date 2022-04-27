//! Module handling the DNS resolving operations
//!
//! We define a list of IP tuples from the `dmarc_rs::ip` crate and implement two methods
//! for resolving the IP into names.  One is `simple_solve()` which is a straightforward sequential
//! solver, the other one is `parallel_solve()` which is using threads from a pool to implement a
//! worker-based fan-out/fan-in scheme with channels to move data around.
//!
//! Examples:
//! ```rust
//! use dmarc_rs::resolve::IpList;
//!
//! let l = IpList::new();
//! // populate here the list of IP tuples
//!
//! // Use the simple solver
//! let ptr = l.simple_solve();
//! dbg!(&ptr);
//! ```
//! and with the parallel solver:
//! ```rust
//! use dmarc_rs::resolve::IpList;
//! use num_cpus::get_physical;
//!
//! // Get the number of physical cores, I prefer to use this one instead of the potentially
//! // larger total cores because Hyperthreading has some overhead.
//! let njobs = get_physical();
//!
//! let l = IpList::new();
//! // populate here the list of IP tuples
//! // ...
//! // Use the parallel solver
//! let ptr = l.parallel_solve(njobs);
//! dbg!(&ptr);
//! ```
//!

// Our crates
//
use crate::ip::Ip;

// Std library
//
use std::error::Error;
use std::thread;
use std::sync::mpsc::{channel, Receiver};

// External crates
//
use anyhow::Result;
use threadpool::ThreadPool;

/// List of IP tuples.
///
/// This is now a distinct type instead of an alias, it is easier to add stuff into it.
///
#[derive(Debug)]
pub struct IpList {
    pub list: Vec<Ip>,
}

/// Implement the Default Trait.
///
impl Default for IpList {
    fn default() -> Self {
        Self::new()
    }
}

/// Methods for IPList
///
impl IpList {
    /// Basic new()
    ///
    /// Example:
    /// ```
    /// # use dmarc_rs::resolve::IpList;
    /// let l = IpList::new();
    /// assert!(l.list.is_empty());
    /// ```
    ///
    pub fn new() -> Self {
        IpList {
            list: vec!(),
        }
    }

    /// Convert a list of IP into names with multiple threads
    ///
    /// It uses a function to fillin the input channel then fan_out/fan_in to fill the result
    /// list.
    ///
    /// Example:
    /// ```no_run
    /// # use dmarc_rs::ip::Ip;
    /// # use dmarc_rs::resolve::IpList;
    /// let mut l = IpList::new();
    /// l.push(Ip::new( "1.1.1.1"));
    /// l.push(Ip::new( "2606:4700:4700::1111"));
    /// l.push(Ip::new( "192.0.2.1"));
    ///
    /// let ptr = l.parallel_solve(4);
    /// ```
    ///
    pub fn parallel_solve(&self, njobs: usize) -> IpList {
        let mut full = IpList::new();
        let s = self.len();

        let pool = ThreadPool::new(njobs);
        let rx_gen = self.queue().unwrap();
        let rx_out = fan_out(rx_gen, pool, s).unwrap();
        for ip in fan_in(rx_out).unwrap() {
            full.push(ip);
        }
        dbg!(&full);
        full
    }

    /// Take all values from the list and send them into a queue
    ///
    fn queue(&self) -> Result<Receiver<Ip>> {
        let (tx, rx) = channel();

        // construct a copy of the list
        let all: Vec<Ip> = self.list.iter().map(|ip| ip.clone()).collect();

        // use that copy to send over
        thread::spawn(move || {
            for ip in all.iter() {
                tx.send(ip.clone()).expect("can not queue")
            }
        });
        Ok(rx)
    }

    /// Simple and straightforward sequential solver
    ///
    /// Example:
    /// ```
    /// # use dmarc_rs::resolve::{IpList};
    /// # use dmarc_rs::ip::Ip;
    /// let mut l = IpList::new();
    /// l.push(Ip::new( "1.1.1.1"));
    /// l.push(Ip::new( "2606:4700:4700::1111"));
    /// l.push(Ip::new( "192.0.2.1"));
    ///
    /// let ptr = l.simple_solve();
    /// ```
    ///
    pub fn simple_solve(&self) -> Self {
        let mut r = IpList::new();

        for ip in self.list.iter() {
            let ip = ip.solve();
            r.push(ip.clone());
        }
        r
    }

    /// Helper fn to add IP to a list
    ///
    /// Example:
    /// ```
    /// # use dmarc_rs::ip::Ip;
    /// # use dmarc_rs::resolve::IpList;
    /// let mut l = IpList::new();
    /// l.push(Ip::new("1.1.1.1"));
    /// ```
    ///
    pub fn push(&mut self, ip: Ip) {
        self.list.push(ip);
    }

    /// Implement len() or IPList
    ///
    /// Example:
    /// ```
    /// # use dmarc_rs::ip::Ip;
    /// # use dmarc_rs::resolve::IpList;
    /// let mut l = IpList::new();
    /// l.push(Ip::new("1.1.1.1"));
    /// println!("length of l is {}", l.len())
    /// ```
    ///
    pub fn len(&self) -> usize {
        self.list.len()
    }
}

/// We want to iterate on IPList as if were on IPList.list itself.
///
impl Iterator for IpList {
    type Item = Ip;

    fn next(&mut self) -> Option<Self::Item> {
        match self.list.iter().next() {
            Some(ip) => Some(ip.clone()),
            None => None,
        }
    }
}

/// Create an `IpList` from an iterator of `&str`.
///
/// Example:
/// ```
/// use dmarc_rs::resolve::IpList;
///
/// let l = IpList::from([("1.1.1.1", "one.one.one.one"), ("2606:4700:4700::1111", "one.one.one.one")]);
///
/// assert_eq!(2, l.len());
/// ```
///
impl<const N: usize> From<[(&str,&str);N]> for IpList {
    fn from(arr: [(&str, &str); N]) -> Self {
        Self::from_iter(arr)
    }
}

/// Create an `IpList` from an iterator of `(&str,&str)` tuples.
///
/// Example:
/// ```
/// use dmarc_rs::resolve::IpList;
///
/// let l = IpList::from(["1.1.1.1", "2606:4700:4700::1111", "192.0.2.1"]);
///
/// assert_eq!(3, l.len());
/// ```
///
impl<const N: usize> From<[&str;N]> for IpList {
    fn from(arr: [&str; N]) -> Self {
        Self::from_iter(arr)
    }
}

/// Actual implementation of `IpList::from_iter` for an array of `&str`
///
impl<'a> FromIterator<&'a str> for IpList
{
    fn from_iter<T: IntoIterator<Item=&'a str>>(iter: T) -> Self {
        let mut ipl = IpList::new();

        for ip in iter {
            ipl.push(Ip::new(ip))
        }
        ipl
    }
}

/// Actual implementation of `IpList::from_iter` for an array of `(&str,&str)` tuples
///
impl<'a> FromIterator<(&'a str,&'a str)> for IpList
{
    fn from_iter<T: IntoIterator<Item=(&'a str, &'a str)>>(iter: T) -> Self {
        let mut ipl = IpList::new();

        for (ip, name) in iter {
            ipl.push(Ip::from((ip, name)))
        }
        ipl
    }
}

// --- Private functions

/// Start enough workers to resolve IP into PTR.
///
fn fan_out(rx_gen: Receiver<Ip>, pool: ThreadPool, njobs: usize) -> Result<Receiver<Ip>, Box<dyn Error>> {
    let (tx, rx) = channel();

    for _ in 0..njobs {
        let tx = tx.clone();
        let n = rx_gen.recv().unwrap();
        pool.execute(move || {
            let r = n.solve();
            tx.send(r)
                .expect("waiting channel");
        });
    }
    Ok(rx)
}

/// Gather all results into an output channel
///
fn fan_in(rx_out: Receiver<Ip>) -> Result<Receiver<Ip>, Box<dyn Error>> {
    let (tx, rx) = channel();
    thread::spawn(move || {
        for ip in rx_out.iter() {
            tx.send(ip)
                .expect("can not send");
        }
    });
    Ok(rx)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ip::Ip;

    #[test]
    fn test_push() {
        let mut l = IpList::new();

        l.push(Ip::new("9.9.9.9"));
        l.push(Ip::new("1.0.0.1"));

        assert_eq!(2, l.len());
        assert_eq!("9.9.9.9", l.list[0].ip.to_string());
    }

    #[test]
    fn test_parallel_solve_empty() {
        let a = IpList::new();

        let r = a.parallel_solve(num_cpus::get_physical());
        assert!(r.list.is_empty())
    }

    #[test]
    fn test_simple_solve_empty() {
        let a = IpList::new();
        let r = a.simple_solve();

        assert!(r.list.is_empty())
    }

    #[test]
    fn test_simple_solve_ok() {
        let l = IpList::from(["1.1.1.1", "2606:4700:4700::1111", "192.0.2.1"]);

        let ptr = l.simple_solve();

        assert_eq!(l.len(), ptr.len());
        assert_eq!(ptr.list[0].name.to_string(), "one.one.one.one");
        assert_eq!(ptr.list[1].name.to_string(), "one.one.one.one");
        assert_eq!(ptr.list[2].name.to_string(), "some.host.invalid");
    }

    #[test]
    fn test_parallel_solve_ok() {
        let l = IpList::from(["1.1.1.1", "2606:4700:4700::1111", "192.0.2.1"]);

        let ptr = l.parallel_solve(num_cpus::get_physical());

        assert_eq!(ptr.list[0].name.to_string(), "one.one.one.one");
        assert_eq!(ptr.list[1].name.to_string(), "one.one.one.one");
        assert_eq!(ptr.list[2].name.to_string(), "some.host.invalid");
    }

}

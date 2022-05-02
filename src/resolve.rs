//! Module handling the DNS resolving operations
//!
//! We define a list of IP tuples from the `dmarc_rs::ip` crate and implement two methods
//! for resolving the IP into names.  One is `simple_solve()` which is a straightforward sequential
//! solver, the other one is `parallel_solve()` which is using threads from a pool to implement a
//! worker-based fan-out/fan-in scheme with channels to move data around.
//!
//! Examples:
//! ```rust
//! # use dmarc_rs::resolve::IpList;
//! let l = IpList::new();
//!
//! // populate here the list of IP tuples
//!
//! // Use the simple solver
//! let ptr = l.simple_solve();
//! dbg!(&ptr);
//! ```
//! and with the parallel solver:
//! ```rust
//! # use dmarc_rs::resolve::IpList;
//! use num_cpus::get_physical;
//!
//! // Get the number of physical cores, I prefer to use this one instead of the potentially
//! // larger total cores because Hyperthreading has some overhead.
//! let njobs = get_physical();
//!
//! let l = IpList::new();
//!
//! // populate here the list of IP tuples
//! // ...
//!
//! // Use the parallel solver
//! let ptr = l.parallel_solve(njobs);
//! ```
//!
//! BUGS: this version only handle one name per IP (whatever is returned by `lookup_addr()`.
//!

// Our crates
//
use crate::ip::Ip;

// Std library
//
use std::error::Error;
use std::ops::{Index, IndexMut};
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
#[derive(Debug, PartialEq)]
pub struct IpList {
    /// A growable list of `Ip`.
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
    /// assert!(l.is_empty());
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
    /// let l = IpList::from(["1.1.1.1", "2606:4700:4700::1111", "192.0.2.1"]);
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
        let all: Vec<Ip> = self.list.clone();

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
    /// let l = IpList::from(["1.1.1.1", "2606:4700:4700::1111", "192.0.2.1"]);
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

    /// Implement is_empty() as a complement to len()
    ///
    /// Example:
    /// ```
    /// # use dmarc_rs::resolve::IpList;
    /// let ipl = IpList::from(["1.0.0.1", "1.1.1.1"]);
    ///
    /// assert!(!ipl.is_empty());
    /// ```
    ///
    pub fn is_empty(&self) -> bool {
        self.list.is_empty()
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
/// # use dmarc_rs::resolve::IpList;
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
/// # use dmarc_rs::resolve::IpList;
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
/// Example:
/// ```
/// # use dmarc_rs::ip::Ip;
/// # use dmarc_rs::resolve::IpList;
/// let l = IpList::from(["1.1.1.1", "2606:4700:4700::1111", "192.0.2.1"]);
/// ```
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
/// Example:
/// ```
/// # use dmarc_rs::resolve::IpList;
/// let l = IpList::from([
///     ("1.1.1.1", "one.one.one.one"),
///     ("2606:4700:4700::1111", "one.one.one.one"),
///     ("192.0.2.1", "some.host.invalid")
/// ]);
/// ```
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

/// Implement Index on IpList for assessing list.
///
/// Example:
/// ```
/// # use dmarc_rs::resolve::IpList;
/// let l = IpList::from(["1.1.1.1", "2606:4700:4700::1111", "192.0.2.1"]);
///
/// println!("{:?}", l[0]);
/// ```
///
impl Index<usize> for IpList {
    type Output = Ip;

    fn index(&self, index: usize) -> &Self::Output {
        &self.list[index]
    }
}

/// Implement IndexMut on IpList for assessing list as mutable objects.
///
/// Example:
/// ```
/// # use dmarc_rs::ip::Ip;
/// use dmarc_rs::resolve::IpList;
/// let mut l = IpList::from(["1.1.1.1", "2606:4700:4700::1111", "192.0.2.1"]);
/// l[0] = Ip::new("9.9.9.9");
/// println!("{:?}", l[0]);
/// ```
///
impl IndexMut<usize> for IpList {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.list[index]
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
    use std::net::IpAddr;

    #[test]
    fn test_push() {
        let mut l = IpList::new();

        l.push(Ip::new("9.9.9.9"));
        l.push(Ip::new("1.0.0.1"));

        assert_eq!(2, l.len());
        assert_eq!("9.9.9.9", l.list[0].ip.to_string());
    }

    #[test]
    fn test_len() {
        let mut ipl = IpList::new();

        assert_eq!(0, ipl.len());

        ipl.push(Ip::new("1.0.0.1"));
        assert_eq!(1, ipl.len());
    }

    #[test]
    fn test_is_empty() {
        let mut ipl = IpList::new();

        assert!(ipl.is_empty());

        ipl.push(Ip::new("1.0.0.1"));
        assert_eq!(false, ipl.is_empty());
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
        assert_eq!("one.one.one.one", ptr[0].name.to_string());
        assert_eq!("one.one.one.one", ptr[1].name.to_string());
        assert_eq!("some.host.invalid", ptr[2].name.to_string());
    }

    #[test]
    fn test_parallel_solve_ok() {
        let l = IpList::from(["1.1.1.1", "2606:4700:4700::1111", "192.0.2.1"]);

        let ptr = l.parallel_solve(num_cpus::get_physical());

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
    fn test_from_array_str() {
        let l = IpList{list: vec![Ip::new("1.1.1.1"), Ip::new("2606:4700:4700::1111"), Ip::new("192.0.2.1")]};
        let l2 = IpList::from(["1.1.1.1", "2606:4700:4700::1111", "192.0.2.1"]);

        assert_eq!(l, l2);
    }

    #[test]
    fn test_from_array_tuples() {
         use std::net::IpAddr;

         let l = IpList{list: vec![
             Ip{ip: "1.1.1.1".parse::<IpAddr>().unwrap(), name: "one.one.one.one".into()},
             Ip{ip: "2606:4700:4700::1111".parse::<IpAddr>().unwrap(), name: "one.one.one.one".into()},
             Ip{ip: "192.0.2.1".parse::<IpAddr>().unwrap(), name: "some.host.invalid".into()},
         ]};
         let l2 = IpList::from([("1.1.1.1", "one.one.one.one"),
             ("2606:4700:4700::1111", "one.one.one.one"),
             ("192.0.2.1", "some.host.invalid")
         ]);

         assert_eq!(l, l2);
    }

    #[test]
    fn test_partial_eq() {
        let a = IpList::from([("1.1.1.1", "one.one.one.one")]);
        let b = IpList::from([("1.1.1.1", "one.one.one.one")]);

        assert_eq!(a, b);
    }

    #[test]
    fn test_index() {
        let l = IpList::from(["1.1.1.1", "2606:4700:4700::1111", "192.0.2.1"]);
        assert_eq!(Ip::new("1.1.1.1").ip, l[0].ip);
    }

    #[test]
    fn test_index_mut() {
        let mut l = IpList::from(["1.1.1.1", "2606:4700:4700::1111", "192.0.2.1"]);
        l[0] = Ip::new("9.9.9.9");
        assert_eq!("9.9.9.9".parse::<IpAddr>().unwrap(), l[0].ip);
    }
}

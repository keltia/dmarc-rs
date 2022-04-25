//! Module handling the DNS interactions
//!

// Our crates
//
use crate::ip::IP;

// External crates
//
use dns_lookup::lookup_addr;

/// List of IP tuples.
pub type IPList = Vec<IP>;

    /// Convert a list of IP into names with multiple threads
    ///
    /// It uses a function to fillin the input channel then fan_out/fan_in to fill the result
    /// list.
    ///
    /// Example:
    ///
    pub fn parallel_solve(self, njobs: usize) -> IPList {
        let mut full = IPList::new();
        let s = self.list.len();

        let pool = ThreadPool::new(njobs);
        let rx_gen = queue(self).unwrap();
        let rx_out = fan_out(rx_gen, pool, s).unwrap();
        for ip in fan_in(rx_out).unwrap() {
            full.list.push(ip);
        }
        dbg!(&full);
        full
    }

    pub fn push(mut self, ip: IP) {
        self.list.push(ip);
    }

    /// Simple and straightforward sequential solver
    ///
    /// Example:
    /// ```
    /// # use dmarc_rs::resolve::{simple_solve,IP,IPList};
    ///
    /// let mut l = IPList::new();
    /// l.push(IP::new( "1.1.1.1"));
    /// l.push(IP::new( "2606:4700:4700::1111"));
    /// l.push(IP::new( "192.0.2.1"));
    ///
    /// let ptr = l.simple_solve();
    /// ```
    ///
    pub fn simple_solve(self) -> Self {
        let mut r = IPList::new();

        for ip in self.list.iter() {
            let ip = ip.solve();
            r.list.push(ip.clone());
        }
        r
    }

    /// Helper fn to add IP to a list
    ///
    /// Example:
    /// ```
    /// # use dmarc_rs::ip::IP;
    /// # use dmarc_rs::resolve::IPList;
    ///
    /// let mut l = IPList::new();
    /// l.push(IP::new("1.1.1.1"));
    /// ```
    pub fn push(mut self, ip: IP) {
        self.list.push(ip);
    }
}

use std::error::Error;
use std::thread;
use std::sync::mpsc::{channel, Receiver};
use threadpool::ThreadPool;

fn queue(l: IPList) -> Result<Receiver<IP>> {
    let (tx, rx) = channel();

    thread::spawn(move || {
        for ip in l.list.iter() {
            tx.send(ip.clone()).expect("can not queue")
        }
    });
    Ok(rx)
}

fn fan_out(rx_gen: Receiver<IP>, pool: ThreadPool, njobs: usize) -> Result<Receiver<IP>, Box<dyn Error>> {
    let (tx, rx) = channel();

    for _ in 0..njobs {
        let tx = tx.clone();
        let n = rx_gen.recv().unwrap();
        pool.execute(move || {
            let name = lookup_addr(&n.ip).unwrap();
            let r = IP { ip: n.ip, name: name };
            tx.send(r)
                .expect("waiting channel");
        });
    }
    Ok(rx)
}

fn fan_in(rx_out: Receiver<IP>) -> Result<Receiver<IP>, Box<dyn Error>> {
    let (tx, rx) = channel();
    thread::spawn(move || {
        for ip in rx_out.iter() {
            tx.send(ip)
                .expect("can not send");
        }
    });
    Ok(rx)
}

/// Simple and straightforward sequential solver
///
/// Example:
/// ```
/// # use dmarc_rs::resolve::{simple_solve,IP,IPList};
///
/// let mut l = IPList::new();
/// l.push(IP::new( "1.1.1.1"));
/// l.push(IP::new( "2606:4700:4700::1111"));
/// l.push(IP::new( "192.0.2.1"));
///
/// let ptr = simple_solve(l);
/// ```
///
pub fn simple_solve(l: IPList) -> IPList {
    let mut r = IPList::new();

    for ip in l {
        let ip = ip.solve();
        r.push(ip.clone());
    }
    r
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ip::IP;
    use std::net::IpAddr;

    #[test]
    fn test_parallel_solve_empty() {
        let a = IPList::new();

        assert!(parallel_solve(a).is_empty())
    }

    #[test]
    fn test_simple_solve_empty() {
        let a = IPList::new();

        assert!(simple_solve(a).is_empty())
    }

    #[test]
    fn test_simple_solve_ok() {
        let mut l = IPList::new();

        l.push(IP::new ( "1.1.1.1"));
        l.push(IP::new ( "2606:4700:4700::1111"));
        l.push(IP::new ( "192.0.2.1"));

        let ptr = simple_solve(l);

        assert_eq!(ptr[0].name.to_string(), "one.one.one.one");
        assert_eq!(ptr[1].name.to_string(), "one.one.one.one");
        assert_eq!(ptr[2].name.to_string(), "some.host.invalid");
    }

    #[test]
    fn test_new_from_tuple() {
        let exp = IP { ip: "1.1.1.1".parse::<IpAddr>().unwrap(), name: "one.one.one.one".into() };

        let t = IP::from(("1.1.1.1", "one.one.one.one"));

        assert_eq!(exp, t);
    }
}

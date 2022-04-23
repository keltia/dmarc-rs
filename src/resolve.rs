//! Module handling the DNS interactions
//!

// Std library
//
use std::net::IpAddr;

// External crates
//
use dns_lookup::lookup_addr;

/// Individual IP/name tuple
#[derive(Clone, Debug, PartialEq)]
pub struct IP {
    /// IP, can be IPv4 or IPv6
    pub ip: IpAddr,
    /// hostname.
    pub name: String,
}

/// Implement a few helpers functions for IP
impl IP {
    /// Create a new tuple with empty name.
    ///
    /// `new()` will panic with an invalid IPv{4,6} address
    ///
    /// Example:
    /// ```rust
    /// # use dmarc_rs::resolve::IP;
    ///
    /// let ip = IP::new("1.1.1.1")
    /// # ;
    /// ```
    ///
    pub fn new(s: &str) -> Self {
        IP {
            ip: s.parse::<IpAddr>().unwrap(),
            name: "".into(),
        }
    }

    /// Get the PTR value for the given IP
    ///
    /// Examples:
    /// ```rust,no_run
    /// # use dmarc_rs::resolve::IP;
    ///
    /// let ptr = IP::new("1.1.1.1").solve();
    /// assert_eq!("one.one.one.one", ptr.name)
    /// # ;
    /// ```
    ///
    /// If there is no PTR, returns a specific valid hostname.
    ///
    /// Example:
    /// ```rust,no_run
    /// # use dmarc_rs::resolve::IP;
    ///
    /// let ptr = IP::new("192.0.2.1").solve();
    /// assert_eq!("some.host.invalid", ptr.name)
    /// # ;
    /// ```
    ///
    pub fn solve(&self) -> Self {
        let ip = self.ip;
        let name = match lookup_addr(&ip) {
            Ok(nm) =>  {
                // No PTR, force one
                if ip.to_string() == nm {
                    "some.host.invalid".into()
                } else {
                    nm
                }
            },
            Err(e) => e.to_string(),
        };
        IP { ip, name }
    }
}

/// Create a new IP from a tuple with all fields
///
/// Example:
/// ```
/// # use std::net::IpAddr;
/// use dmarc_rs::resolve::IP;
/// let t = IP::from(("1.1.1.1", "one.one.one.one"));
///
/// assert_eq!("1.1.1.1".parse::<IpAddr>().unwrap(), t.ip);
/// assert_eq!("one.one.one.one", &t.name);
/// ```
///
impl<'a> From<(&'a str, &'a str)> for IP {
    fn from((ip, name): (&'a str, &'a str)) -> Self {
        IP { ip: ip.parse::<IpAddr>().unwrap(), name: name.into() }
    }
}

/// List of IP tuples.
pub type IPList = Vec<IP>;

/// Convert a list of IP into names with multiple threads
///
/// Example:
///
pub fn parallel_solve(l: IPList) -> IPList {
    let full = IPList::new();

    dbg!(full)
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
    use rstest::rstest;

    #[rstest]
    #[case("0.0.0.0")]
    #[case("127.0.0.1")]
    #[case("1.2.3.4")]
    #[case("10.0.0.1")]
    #[case("172.16.1.1")]
    #[case("192.168.1.1")]
    #[case("::127.0.0.1")]
    #[case("::face:b00c")]
    #[case("3ffe::a:b:c:d:e")]
    fn test_ip_new_ok(#[case] s: &str) {
        let a1 = IP::new(s);
        assert!(a1.name.is_empty());
        assert_eq!(s.parse::<IpAddr>().unwrap(), a1.ip)
    }

    #[rstest]
    #[case("333.0.0.0")]
    #[case("127.333.0.1")]
    #[case("1.2.333.4")]
    #[case("10.0.0.555")]
    #[case("foobar")]
    #[case("::blah:blah::.168.1.1")]
    #[should_panic]
    fn test_ip_new_nok(#[case] s: &str) {
        let _a1 = IP::new(s);
    }

    #[rstest]
    #[case("1.1.1.1", "one.one.one.one")]
    #[case("2606:4700:4700::1111", "one.one.one.one")]
    #[case("192.0.2.1", "some.host.invalid")]
    fn test_ip_solve(#[case] s: &str, #[case] p: &str) {
        let ptr = IP::new(s).solve();
        assert_eq!(s.parse::<IpAddr>().unwrap(), ptr.ip);
        assert_eq!(p.to_string(), ptr.name);
    }

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

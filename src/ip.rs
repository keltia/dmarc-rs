//! Helper module to deal with tuples of IP/names.
//!
//! We define IP as a tuple containing the `IpAddr` and the name (initially empty of course).
//! To facilitate manipulations, we also define `from` to magically convert tuples of strings
//! into an `IP`.
//!
//! Example:
//! ```
//! use dmarc_rs::ip::Ip;
//!
//! let me = Ip::new("127.0.0.1");
//! ```
//! or
//! ```
//! use dmarc_rs::ip::Ip;
//!
//! let me = Ip::from(("::1", "localhost"));
//! ```
//!
//! The `solve()`  method does the A/AAAA to PTR conversion with a twist: if the IP can't be
//! resolved it returns "some.host.invalid".
//!

// Std library
//
use std::net::IpAddr;

// External crates
//
use dns_lookup::lookup_addr;

/// Individual IP/name tuple
#[derive(Clone, Debug, PartialEq)]
pub struct Ip {
    /// IP, can be IPv4 or IPv6
    pub ip: IpAddr,
    /// hostname.
    pub name: String,
}

/// Implement a few helpers functions for IP
impl Ip {
    /// Create a new tuple with empty name.
    ///
    /// `new()` will panic with an invalid IPv{4,6} address
    ///
    /// Example:
    /// ```rust
    /// # use dmarc_rs::ip::Ip;
    ///
    /// let ip = Ip::new("1.1.1.1")
    /// # ;
    /// ```
    ///
    pub fn new(s: &str) -> Self {
        Ip {
            ip: s.parse::<IpAddr>().unwrap(),
            name: "".into(),
        }
    }

    /// Get the PTR value for the given IP
    ///
    /// Examples:
    /// ```rust,no_run
    /// # use dmarc_rs::ip::Ip;
    ///
    /// let ptr = Ip::new("1.1.1.1").solve();
    /// assert_eq!("one.one.one.one", ptr.name)
    /// # ;
    /// ```
    ///
    /// If there is no PTR, returns a specific valid hostname.
    ///
    /// Example:
    /// ```rust,no_run
    /// # use dmarc_rs::ip::Ip;
    ///
    /// let ptr = Ip::new("192.0.2.1").solve();
    /// assert_eq!("some.host.invalid", ptr.name)
    /// # ;
    /// ```
    ///
    pub fn solve(&self) -> Self {
        let ip = self.ip;
        let name = match lookup_addr(&ip) {
            Ok(nm) => {
                // No PTR, force one
                if ip.to_string() == nm {
                    "some.host.invalid".into()
                } else {
                    nm
                }
            }
            Err(e) => e.to_string(),
        };
        Ip { ip, name }
    }
}

impl From<&str> for Ip {
    fn from(s: &str) -> Self {
        Ip::new(s)
    }
}

/// Create a new IP from a tuple with all fields
///
/// Example:
/// ```
/// # use std::net::IpAddr;
/// use dmarc_rs::ip::Ip;
///
/// let t = Ip::from(("1.1.1.1", "one.one.one.one"));
///
/// assert_eq!("1.1.1.1".parse::<IpAddr>().unwrap(), t.ip);
/// assert_eq!("one.one.one.one", &t.name);
/// ```
///
impl From<(&str, &str)> for Ip {
    fn from((ip, name): (&str, &str)) -> Self {
        Ip {
            ip: ip.parse::<IpAddr>().unwrap(),
            name: name.into(),
        }
    }
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
        let a1 = Ip::new(s);
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
        let _a1 = Ip::new(s);
    }

    #[rstest]
    #[case("1.1.1.1", "one.one.one.one")]
    #[case("2606:4700:4700::1111", "one.one.one.one")]
    #[case("192.0.2.1", "some.host.invalid")]
    fn test_ip_solve(#[case] s: &str, #[case] p: &str) {
        let ptr = Ip::new(s).solve();
        assert_eq!(s.parse::<IpAddr>().unwrap(), ptr.ip);
        assert_eq!(p.to_string(), ptr.name);
    }

    #[test]
    fn test_new_from_tuple() {
        let exp = Ip {
            ip: "1.1.1.1".parse::<IpAddr>().unwrap(),
            name: "one.one.one.one".into(),
        };

        let t = Ip::from(("1.1.1.1", "one.one.one.one"));

        assert_eq!(exp, t);
    }
}
//! Module handling the DNS interactions
//!

// Std library
//
use std::net::IpAddr;

/// Individual IP/name tuple
#[derive(Clone, Debug)]
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
}

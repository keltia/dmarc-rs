//! Module defining the list of IP address we get from the XML file
//!
//! **NOTE**  all the resolving part has been removed from that part due to issues
//! with implementing support for multiple resolvers (useful for testing).

// Our crates
//
use crate::ip::Ip;

// Std library
//
use std::ops::{Index, IndexMut};

// External crates
//

/// List of IP tuples.
///
/// This is a wrapper type instead of an alias, it is easier to add stuff into it.  The inner list
/// is accessible through `.0`.
///
/// We define the usual set of methods to facilitate initialisation and handling of the inner
/// list of `Ip` inside.
///
/// We also define a large set of operations over `IpList` in order to facilitate manipulation
/// with iterators, creating from iterators, sorting, indexed access, etc.
///
#[derive(Clone, Debug, Eq, PartialOrd, Ord, PartialEq)]
pub struct IpList(Vec<Ip>);

/// Implement the Default Trait.
///
impl Default for IpList {
    fn default() -> Self {
        Self::new()
    }
}

impl IpList {
    /// Basic new()
    ///
    /// Example:
    /// ```
    /// # use dmarc_rs::iplist::IpList;
    /// let l = IpList::new();
    /// assert!(l.is_empty());
    /// ```
    ///
    #[inline]
    pub fn new() -> Self {
        IpList(vec![])
    }

    /// Helper fn to add IP to a list
    ///
    /// Example:
    /// ```
    /// # use dmarc_rs::ip::Ip;
    /// # use dmarc_rs::iplist::IpList;
    /// let mut l = IpList::new();
    /// l.push(Ip::new("1.1.1.1"));
    /// ```
    ///
    #[inline]
    pub fn push(&mut self, ip: Ip) {
        self.0.push(ip);
    }

    /// Implement len() or IPList
    ///
    /// Example:
    /// ```
    /// # use dmarc_rs::ip::Ip;
    /// # use dmarc_rs::iplist::IpList;
    /// let mut l = IpList::new();
    /// l.push(Ip::new("1.1.1.1"));
    /// println!("length of l is {}", l.len())
    /// ```
    ///
    #[inline]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Implement is_empty() as a complement to len()
    ///
    /// Example:
    /// ```
    /// # use dmarc_rs::iplist::IpList;
    /// let ipl = IpList::from(["1.0.0.1", "1.1.1.1"]);
    ///
    /// assert!(!ipl.is_empty());
    /// ```
    ///
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Implement `sort()` directly on `IpList`
    ///
    /// Example:
    /// ```
    /// # use dmarc_rs::iplist::IpList;
    /// let mut ipl = IpList::from(["224.0.0.1", "1.0.0.1", "2.3.4.5", "1.1.1.1", "192.0.2.1"]);
    ///
    /// println!("{:?}", ipl.sort());
    /// ```
    ///
    #[inline]
    pub fn sort(&mut self) {
        self.0.sort();
    }
}

/// Implement `IntoIterator` for `IpList` by calling the inner `into_iter()`.
///
/// Example:
/// ```
/// # use dmarc_rs::ip::Ip;
/// use dmarc_rs::iplist::IpList;
/// let l = IpList::from(["1.1.1.1", "2606:4700:4700::1111"]);
///
/// let s: IpList = l.into_iter().map(|ip| Ip::new("0.0.0.0")).collect();
/// assert_eq!(2, s.len());
/// ```
///
impl IntoIterator for IpList {
    type Item = Ip;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    /// Iterate over the list of Ip, consuming the list
    ///
    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

/// Create an `IpList` from an iterator of `&str`.
///
impl<const N: usize> From<[(&str, &str); N]> for IpList {
    /// Used as a shortcut to `from_iter()`
    ///
    /// Example:
    /// ```
    /// # use dmarc_rs::iplist::IpList;
    /// let l = IpList::from([("1.1.1.1", "one.one.one.one"), ("2606:4700:4700::1111", "one.one.one.one")]);
    ///
    /// assert_eq!(2, l.len());
    /// ```
    ///
    #[inline]
    fn from(arr: [(&str, &str); N]) -> Self {
        Self::from_iter(arr)
    }
}

/// Create an `IpList` from an iterator of `(&str,&str)` tuples.
///
impl<const N: usize> From<[&str; N]> for IpList {
    /// Used as a shortcut to `from_iter()`
    ///
    /// Example:
    /// ```
    /// # use dmarc_rs::iplist::IpList;
    /// let l = IpList::from(["1.1.1.1", "2606:4700:4700::1111", "192.0.2.1"]);
    ///
    /// assert_eq!(3, l.len());
    /// ```
    ///
    #[inline]
    fn from(arr: [&str; N]) -> Self {
        Self::from_iter(arr)
    }
}

impl From<Ip> for IpList {
    /// Create an `IpList` from a single `Ip`.
    ///
    fn from(ip: Ip) -> Self {
        Self(vec![ip])
    }
}

/// Actual implementation of `IpList::from_iter` for an array of `&str`
///
impl<'a> FromIterator<&'a str> for IpList {
    ///
    /// Example:
    /// ```
    /// # use dmarc_rs::ip::Ip;
    /// # use dmarc_rs::iplist::IpList;
    /// let l = IpList::from(["1.1.1.1", "2606:4700:4700::1111", "192.0.2.1"]);
    /// ```
    ///
    fn from_iter<T: IntoIterator<Item = &'a str>>(iter: T) -> Self {
        let mut ipl = IpList::new();

        for ip in iter {
            ipl.push(Ip::new(ip))
        }
        ipl
    }
}

/// Actual implementation of `IpList::from_iter` for an array of `(&str,&str)` tuples
///
impl<'a> FromIterator<(&'a str, &'a str)> for IpList {
    ///
    /// Example:
    /// ```
    /// # use dmarc_rs::iplist::IpList;
    /// let l = IpList::from([
    ///     ("1.1.1.1", "one.one.one.one"),
    ///     ("2606:4700:4700::1111", "one.one.one.one"),
    ///     ("192.0.2.1", "some.host.invalid")
    /// ]);
    /// ```
    ///
    #[inline]
    fn from_iter<T: IntoIterator<Item = (&'a str, &'a str)>>(iter: T) -> Self {
        let mut ipl = IpList::new();

        for (ip, name) in iter {
            ipl.push(Ip::from((ip, name)))
        }
        ipl
    }
}

/// Actual implementation of `IpList::from_iter` for an array of `Ip`, enabling `collect()` support
///
impl FromIterator<Ip> for IpList {
    /// Example:
    /// ```
    /// # use dmarc_rs::ip::Ip;
    /// # use dmarc_rs::iplist::IpList;
    /// let l = IpList::from(["1.1.1.1", "2606:4700:4700::1111"]);
    ///
    /// let s: IpList = l.into_iter().map(|_| Ip::new("0.0.0.0")).collect();
    /// assert_eq!(2, s.len());
    /// assert_eq!("0.0.0.0".to_string(), s[0].ip.to_string());
    /// ```
    ///
    #[inline]
    fn from_iter<I: IntoIterator<Item = Ip>>(iter: I) -> Self {
        let mut ipl = IpList::new();
        for ip in iter {
            ipl.push(ip);
        }
        ipl
    }
}

/// Implement `Index` on `IpList` for accessing list elements.
///
impl Index<usize> for IpList {
    type Output = Ip;

    /// Example:
    /// ```
    /// # use dmarc_rs::iplist::IpList;
    /// let l = IpList::from(["1.1.1.1", "2606:4700:4700::1111", "192.0.2.1"]);
    ///
    /// println!("{:?}", l[0]);
    /// ```
    ///
    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

/// Implement `IndexMut` on `IpList` for accessing list elements as mutable objects.
///
impl IndexMut<usize> for IpList {
    /// Example:
    /// ```
    /// # use dmarc_rs::ip::Ip;
    /// use dmarc_rs::iplist::IpList;
    /// let mut l = IpList::from(["1.1.1.1", "2606:4700:4700::1111", "192.0.2.1"]);
    /// l[0] = Ip::new("9.9.9.9");
    /// println!("{:?}", l[0]);
    /// ```
    ///
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
    }
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
        assert_eq!("9.9.9.9", l.0[0].ip.to_string());
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
    fn test_from_ip() {
        let ip = Ip::new("1.0.0.1");
        let ipl = IpList::from(ip);
        let r = IpList(vec![Ip::new("1.0.0.1")]);

        assert_eq!(r, ipl);
    }

    #[test]
    fn test_from_array_str() {
        let l = IpList(vec![
            Ip::new("1.1.1.1"),
            Ip::new("2606:4700:4700::1111"),
            Ip::new("192.0.2.1"),
        ]);

        let l2 = IpList::from(["1.1.1.1", "2606:4700:4700::1111", "192.0.2.1"]);

        assert_eq!(l, l2);
    }

    #[test]
    fn test_from_array_tuples() {
        use std::net::IpAddr;

        let l = IpList(vec![
            Ip {
                ip: "1.1.1.1".parse::<IpAddr>().unwrap(),
                name: "one.one.one.one".into(),
            },
            Ip {
                ip: "2606:4700:4700::1111".parse::<IpAddr>().unwrap(),
                name: "one.one.one.one".into(),
            },
            Ip {
                ip: "192.0.2.1".parse::<IpAddr>().unwrap(),
                name: "some.host.invalid".into(),
            },
        ]);
        let l2 = IpList::from([
            ("1.1.1.1", "one.one.one.one"),
            ("2606:4700:4700::1111", "one.one.one.one"),
            ("192.0.2.1", "some.host.invalid"),
        ]);

        assert_eq!(l, l2);
    }

    #[test]
    fn test_collect() {
        let l = IpList::from(["1.1.1.1", "2606:4700:4700::1111", "192.0.2.1"]);

        let s: IpList = l.into_iter().map(|ip| ip.clone()).collect();
        assert_eq!(3, s.len());
    }

    #[test]
    fn test_collect_mut() {
        let l = IpList::from(["1.1.1.1", "2606:4700:4700::1111"]);

        let s: IpList = l.into_iter().map(|_| Ip::new("0.0.0.0")).collect();
        assert_eq!(2, s.len());
        assert_eq!("0.0.0.0".to_string(), s[0].ip.to_string());
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

    #[test]
    fn test_iplist_for() {
        let l = IpList::from([
            ("1.1.1.1", "one.one.one.one"),
            ("2606:4700:4700::1111", "one.one.one.one"),
            ("192.0.2.1", "some.host.invalid"),
        ]);
        assert_eq!(3, l.len());

        let r = vec!["1.1.1.1", "2606:4700:4700::1111", "192.0.2.1"];
        assert_eq!(3, r.len());

        let mut ips = vec![];

        for ip in l.into_iter() {
            ips.push(ip.ip.to_string());
        }
        assert_eq!(r, ips);
        dbg!(&ips);
    }

    #[test]
    fn test_sort() {
        let mut ipl = IpList::from(["224.0.0.1", "1.0.0.1", "2.3.4.5", "1.1.1.1", "192.0.2.1"]);
        let s = IpList::from(["1.0.0.1", "1.1.1.1", "2.3.4.5", "192.0.2.1", "224.0.0.1"]);

        ipl.sort();
        assert_eq!(s, ipl);
    }

    #[test]
    fn test_clone() {
        let ipl = IpList::from(["224.0.0.1", "1.0.0.1", "2.3.4.5", "1.1.1.1", "192.0.2.1"]);
        let r = ipl.clone();

        assert_eq!(ipl, r);
    }
}

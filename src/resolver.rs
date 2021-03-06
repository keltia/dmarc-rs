//! Module implementing a generic DNS Resolver trait.
//!
//! It comes with several different implementation for both testing and run-time behaviour change.
//! It uses the mechanism known as [dependency injection][dep-inj].
//!
//! The trait is encapsulated into a new type to avoid exposing internal details of the
//! implementation as described in [this article][jmmv].
//!
//! This system allows for both run-time selection of the resolving module and easier testing
//! for any modules using this mechanism.
//!
//! Here we define 3 main modules:
//!
//! - `NullResolver`: this one just does a copy of the original IP address and the name is the same
//! as the original IP.
//! - `FakeResolver`: this one is for testing mainly as it enables you to `load()` a set of preset
//! values that will be matched and returned.
//! - `RealResolver`: this one is used in the general case (and is the default).  It uses the
//! `lookup_addr()` from the `dns_lookup`  crate.
//!
//! **BUGS** this version only handle **one** name per IP (whatever is returned by `lookup_addr()`).
//!
//! [dep-inj]: https://en.wikipedia.org/wiki/Dependency_injection
//! [jmmv]: https://jmmv.dev/2022/04/rust-traits-and-dependency-injection.html

// Std Library
//
use std::fmt::{Debug, Formatter};
use std::net::IpAddr;
use std::sync::Arc;

// Our crates
//
use crate::ip::Ip;
use crate::iplist::IpList;

// External crates
//
#[cfg(not(test))]
use dns_lookup::lookup_addr;

// When testing, hide the external function to put our own.
// It has to be here and not inside `mod tests` in order to properly shadow the real one.
#[cfg(test)]
fn lookup_addr(_ip: &IpAddr) -> anyhow::Result<String> {
    Ok("foo.bar.invalid".to_string())
}

/// This trait will allow us to override the resolving function during tests & at run-time.
/// It defines a single function that basically get the PTR value from an IP address.  It takes an
/// `Ip` as defined in `crate::dmarc_rs` and returns the same with the `name` field changed to the
/// corresponding resolved name.
///
/// Creating a different resolving mechanism is done simply by creating a new type and implementing
/// the `Resolver` trait.
///
pub trait Resolver {
    /// Get the name associated with the given `Ip`.
    ///
    fn solve(&self, ip: &Ip) -> Ip;
}

/// Opaque type representing the implementation of the `Resolver` trait.
///
#[derive(Clone)]
pub struct Solver(Arc<dyn Resolver + Send + Sync + 'static>);

impl Solver {
    /// Calling the inner implementation of `solve()`
    ///
    pub fn solve(&self, ip: &Ip) -> Ip {
        self.0.solve(ip)
    }
}

/// Enum for selecting the different types of currently supported resolvers.
///
#[derive(Debug)]
pub enum ResType {
    /// For testing, returns a specific value
    Fake,
    /// Returns name == ip
    Null,
    /// The real thing, encapsulating `lookup_addr()`
    Real,
}

impl Default for ResType {
    /// Returns the default resolver (i.e. `Real`).
    #[inline]
    fn default() -> Self {
        ResType::Real
    }
}

/// This is the Null resolver, it returns the IP in the `name` field.
///
pub struct NullResolver;

impl NullResolver {
    /// Returns one instance
    ///
    #[inline]
    pub(crate) fn init() -> Self {
        NullResolver {}
    }
}

impl Resolver for NullResolver {
    /// Implement the `Resolver` trait.
    ///
    #[inline]
    fn solve(&self, ip: &Ip) -> Ip {
        Ip {
            ip: ip.ip,
            name: ip.ip.to_string(),
        }
    }
}

impl Debug for NullResolver {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("nullresolver")
    }
}

/// This is the Fake resolver, for the moment it returns `some.host.invalid`  for all IP.
///
pub struct FakeResolver();

impl FakeResolver {
    /// Returns one instance.
    ///
    #[inline]
    pub(crate) fn init() -> Self {
        FakeResolver {}
    }
}

impl Resolver for FakeResolver {
    /// Implement the `Resolver` trait.
    ///
    #[inline]
    fn solve(&self, ip: &Ip) -> Ip {
        Ip {
            ip: ip.ip,
            name: "some.host.invalid".to_string(),
        }
    }
}

impl Debug for FakeResolver {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("fakeresolver with some.host.invalid")
    }
}

/// This is the real resolver implementation that  resolve IP to hostnames with the system one.
///
pub struct RealResolver;

impl RealResolver {
    /// Returns an instance of the resolver.
    ///
    #[inline]
    pub(crate) fn init() -> Self {
        RealResolver {}
    }
}

impl Resolver for RealResolver {
    /// Implement the `Resolver` trait.
    ///
    #[inline]
    fn solve(&self, ip: &Ip) -> Ip {
        Ip {
            ip: ip.ip,
            name: lookup_addr(&ip.ip).unwrap(),
        }
    }
}

impl Debug for RealResolver {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("realresolver using lookup_addr")
    }
}

/// Create an instance of the Solver type corresponding to one of the resolvers.
///
/// Before using any of these resolver you have to instantiate one of them through `res_init()`.
/// It returns a `Solver` object and you can use `solve()` to get the name.
///
/// Example:
/// ```rust
/// # use dmarc_rs::ip::Ip;
/// # use dmarc_rs::resolver::{res_init, ResType};
/// let res = res_init(ResType::Real);
///
/// let ip = Ip::new("1.1.1.1");
/// // returns an IP
/// let ip = res.solve(&ip);
///
/// println!("{:?}", ip.name);
/// // ==> should print "one.one.one.one"
/// ```
///
#[inline]
pub fn res_init(t: ResType) -> Solver {
    match t {
        ResType::Null => Solver(Arc::from(NullResolver::init())),
        ResType::Fake => Solver(Arc::from(FakeResolver::init())),
        ResType::Real => Solver(Arc::from(RealResolver::init())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::any::{Any, TypeId};

    use rstest::rstest;

    #[rstest]
    #[case(ResType::Fake)]
    #[case(ResType::Null)]
    #[case(ResType::Real)]
    fn test_res_init(#[case] t: ResType) {
        let a = res_init(t);

        assert_eq!(TypeId::of::<Solver>(), a.type_id());
    }

    #[test]
    fn test_null_solve() {
        let a = res_init(ResType::Null);

        let ip = Ip::new("1.1.1.1");
        assert_eq!("1.1.1.1", a.solve(&ip).name);
    }

    #[test]
    fn test_real_solve() {
        let a = res_init(ResType::Real);

        let ip = Ip::new("1.1.1.1");
        assert_eq!("foo.bar.invalid", a.solve(&ip).name);
    }

    #[test]
    fn test_fake_solve() {
        let ipl = IpList::from([("1.1.1.1", "")]);
        let res = res_init(ResType::Fake);

        assert_eq!("some.host.invalid", res.solve(&ipl[0]).name);
    }
}

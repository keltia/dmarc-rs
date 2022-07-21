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
use std::sync::Arc;

// Our crates
//
use crate::res::async_resolve::parallel_solve;
use crate::res::ip::Ip;
use crate::res::iplist::IpList;

// External crates
//
#[cfg(not(test))]
use dns_lookup::lookup_addr;

#[cfg(test)]
use std::net::IpAddr;
use std::thread::sleep;
use std::time::Duration;

// When testing, hide the external function to put our own.
// It has to be here and not inside `mod tests` in order to properly shadow the real one.
#[cfg(test)]
fn lookup_addr(_ip: &IpAddr) -> anyhow::Result<String> {
    Ok("foo.bar.invalid".to_string())
}

/// `resolve()` is the main function call to get all names from the list of `Ip` we get from the
/// XML file.
///
/// Example:
/// ```no_run
/// # use dmarc_rs::res::iplist::IpList;
/// # use dmarc_rs::res::resolver::{res_init, resolve, ResType};
/// let l = IpList::from(["1.1.1.1", "2606:4700:4700::1111", "192.0.2.1"]);
///
/// // Select a resolver
/// let res = res_init(ResType::Real);
///
/// // Using the simple single threaded solver.
/// let ptr = resolve(&l, 1, &res).unwrap();
///
/// // Use the parallel solver with as many threads as the CPU has.
/// let ptr2 = resolve(&l, num_cpus::get(), &res).unwrap();
/// ```
///
pub async fn resolve(
    ipl: &IpList,
    njobs: usize,
    res: &Solver,
) -> Result<IpList, async_std::io::Error> {
    let max_threads = num_cpus::get();

    // Return an error on empty list
    // XXX maybe return the empty list?
    if ipl.is_empty() {
        return Err("Empty list");
    }

    // Put a hard limit on how many parallel thread to the max number of cores (incl.
    // avoid overhead even on modern versions of Hyperthreading).
    //
    if njobs > max_threads {
        return Err(anyhow!("Too many threads"));
    }

    // Bypass the more complex code is IpList has only one element
    if ipl.len() == 1 {
        let ip = res.solve(&ipl[0]);
        return Ok(IpList::from(ip));
    }

    // Call the appropriate one
    //
    match njobs {
        1 => Ok(simple_solve(&ipl, res)),
        _ => {
            let res = parallel_solve(ipl, njobs, res).await?;
            Ok(res)
        }
    }
}

/// Simple and straightforward sequential solver
///
/// Example:
/// ```
/// # use dmarc_rs::res::IpList;
/// # use dmarc_rs::res::Ip;
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
    #[inline]
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
    /// Special one for bench
    Sleep,
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

/// This is the Sleep resolver, for the moment it returns `some.host.invalid`  for all IP.
///
pub struct SleepResolver();

impl SleepResolver {
    /// Returns one instance.
    ///
    #[inline]
    pub(crate) fn init() -> Self {
        SleepResolver {}
    }
}

impl Resolver for SleepResolver {
    /// Implement the `Resolver` trait.
    ///
    #[inline]
    fn solve(&self, ip: &Ip) -> Ip {
        sleep(Duration::from_secs_f32(0.001f32));
        Ip {
            ip: ip.ip,
            name: "some.host.invalid".to_string(),
        }
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

/// Create an instance of the Solver type corresponding to one of the resolvers.
///
/// Before using any of these resolver you have to instantiate one of them through `res_init()`.
/// It returns a `Solver` object and you can use `solve()` to get the name.
///
/// Example:
/// ```rust
/// # use dmarc_rs::res::Ip;
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
        ResType::Sleep => Solver(Arc::from(SleepResolver::init())),
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
    #[case(ResType::Sleep)]
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
        let a = res_init(ResType::Fake);

        let ip = Ip::new("1.1.1.1");
        assert_eq!("some.host.invalid", a.solve(&ip).name);
    }

    #[test]
    fn test_sleep_solve() {
        let a = res_init(ResType::Sleep);

        let ip = Ip::new("1.1.1.1");
        assert_eq!("some.host.invalid", a.solve(&ip).name);
    }
}

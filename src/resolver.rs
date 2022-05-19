//! module implementing a generic DNS Resolver trait with several different
//! implementation for both testing and run-time behaviour change.
//!

// Std Library
//
use std::fmt::{Debug, Formatter};
use std::sync::Arc;

// Our crates
//
use crate::ip::Ip;
use crate::iplist::IpList;

// External crates
//
use dns_lookup::lookup_addr;

/// This trait will allow us to override the resolving function during tests.
///
pub(crate) trait Resolver {
    /// Get the IP 2 PTR for all elements in `IpList`
    fn solve(&self, ip: Ip) -> Ip;
}

/// Opaque type representing the implementation of the `Resolver` trait.
///
#[derive(Clone)]
pub struct Solver(Arc<dyn Resolver + Send + Sync + 'static>);

impl Solver {
    pub fn solve(&self, ip: Ip) -> Ip {
        Ip {
            ip: ip.ip,
            name: "resolved.invalid".to_string(),
        }
    }
}

#[derive(Debug)]
pub enum ResType {
    Null,
    Fake,
    Real,
}

pub struct NullResolver;

impl NullResolver {
    pub(crate) fn init() -> Self {
        NullResolver {}
    }
}

impl Resolver for NullResolver {
    fn solve(&self, ip: Ip) -> Ip {
        ip
    }
}

impl Debug for NullResolver {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("nullresolver")
    }
}

pub struct FakeResolver(IpList);

impl FakeResolver {
    pub(crate) fn init() -> Self {
        FakeResolver(IpList::new())
    }

    pub fn load(&self, ipl: IpList) -> Self {
        let mut r = FakeResolver::init();
        for ip in ipl.into_iter() {
            r.0.push(ip)
        }
        r
    }
}

impl Resolver for FakeResolver {
    fn solve(&self, ip: Ip) -> Ip {
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

pub struct RealResolver;

impl RealResolver {
    pub(crate) fn init() -> Self {
        RealResolver {}
    }
}

impl Resolver for RealResolver {
    fn solve(&self, ip: Ip) -> Ip {
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

pub fn res_init(t: ResType) -> Solver {
    match t {
        ResType::Null => Solver(Arc::from(NullResolver::init())),
        ResType::Fake => Solver(Arc::from(FakeResolver::init())),
        ResType::Real => Solver(Arc::from(RealResolver::init())),
    }
}


//! Benchmark for the sync version of the fan-out/fan-in pattern.
//!
//! Please run with `cargo bench`.
//!

use dmarc_rs::ip::Ip;
use dmarc_rs::iplist::IpList;
use dmarc_rs::resolver::*;

// Std library
//
use std::net::IpAddr;
use std::sync::mpsc::{channel, Receiver};
use std::thread;

// External crates
//
use anyhow::Result;
use criterion::{criterion_group, criterion_main, Criterion};
use threadpool::ThreadPool;

/// **NOTE** the code is reimplemented here because a bench can not import from the binary, only
/// the library.

/// Simple and straightforward sequential solver
///
/// Example:
/// ```
/// # use dmarc_rs::iplist::IpList;
/// # use dmarc_rs::ip::Ip;
/// let l = IpList::from(["1.1.1.1", "2606:4700:4700::1111", "192.0.2.1"]);
///
/// // select a given resolver
/// let res = res_init(ResType::Null);
///
/// let ptr = simple_solve(l, res);
/// ```
///
fn simple_solve(ipl: &IpList, res: &Solver) -> IpList {
    let r: IpList = ipl.clone().into_iter().map(|ip| res.solve(&ip)).collect();
    assert_eq!(ipl.len(), r.len());
    r
}

/// Convert a list of IP into names with multiple threads
///
/// It uses a function to fill in the input channel then fan_out/fan_in to fill the result
/// list.
///
/// Example:
/// ```no_run
/// # use dmarc_rs::ip::Ip;
/// # use dmarc_rs::iplist::IpList;
/// let l = IpList::from(["1.1.1.1", "2606:4700:4700::1111", "192.0.2.1"]);
///
/// // Select a resolver
/// let res = res_init(ResType::Null);
///
/// let ptr = parallel_solve(l, 4, res);
/// ```
///
fn parallel_solve(ipl: &IpList, workers: usize, res: &Solver) -> IpList {
    let mut full = IpList::new();
    let s = ipl.len();

    let pool = ThreadPool::new(workers);
    let rx_gen = queue(ipl).unwrap();
    let rx_out = fan_out(rx_gen, pool, s, res).unwrap();
    for ip in fan_in(rx_out).unwrap() {
        full.push(ip);
    }
    //full.sort();
    assert_eq!(ipl.len(), full.len());
    full
}

/// Take all values from the list and send them into a queue
///
fn queue(ipl: &IpList) -> Result<Receiver<Ip>> {
    let (tx, rx) = channel();

    // construct a copy of the list
    let all = ipl.clone();

    // use that copy to send over
    thread::spawn(move || {
        for ip in all.into_iter() {
            tx.send(ip).expect("can not queue")
        }
    });
    Ok(rx)
}

/// Start enough workers to resolve IP into PTR.
///
fn fan_out(
    rx_gen: Receiver<Ip>,
    pool: ThreadPool,
    njobs: usize,
    res: &Solver,
) -> Result<Receiver<Ip>, Box<dyn std::error::Error>> {
    let (tx, rx) = channel();

    for _ in 0..njobs {
        let tx = tx.clone();
        let n = rx_gen.recv().unwrap();

        let res = res.clone();
        pool.execute(move || {
            let name: Ip = res.solve(&n);
            tx.send(name).expect("waiting channel");
        });
    }
    Ok(rx)
}

/// Gather all results into an output channel
///
fn fan_in(rx_out: Receiver<Ip>) -> Result<Receiver<Ip>, Box<dyn std::error::Error>> {
    let (tx, rx) = channel();
    thread::spawn(move || {
        for ip in rx_out.iter() {
            tx.send(ip).expect("can not send");
        }
    });
    Ok(rx)
}

use fake::{Fake, Faker};

/// Generate a dataset of N fake IpAddr
///
fn setup() -> IpList {
    let mut ipl = IpList::new();
    for _ in 1..100 {
        let ip: IpAddr = Faker.fake();
        ipl.push(Ip {
            ip,
            name: "".to_string(),
        });
    }
    ipl
}

fn resolve_simple(c: &mut Criterion) {
    let res = res_init(ResType::Sleep);

    let ipl = setup();
    let mut r = IpList::new();

    c.bench_function("simple_solve 100", |b| {
        b.iter(|| r = simple_solve(&ipl, &res))
    });

    let _ = r;
}

fn resolve_parallel_1(c: &mut Criterion) {
    let res = res_init(ResType::Sleep);

    let ipl = setup();
    let mut r = IpList::new();

    c.bench_function("parallel_solve 100/1", |b| {
        b.iter(|| r = parallel_solve(&ipl, 1, &res))
    });

    let _ = r;
}

fn resolve_parallel_4(c: &mut Criterion) {
    let res = res_init(ResType::Null);

    let ipl = setup();
    let mut r = IpList::new();

    c.bench_function("parallel_solve 100/4", |b| {
        b.iter(|| r = parallel_solve(&ipl, 4, &res))
    });

    let _ = r;
}

fn resolve_parallel_6(c: &mut Criterion) {
    let res = res_init(ResType::Null);

    let ipl = setup();
    let mut r = IpList::new();

    c.bench_function("parallel_solve 100/6", |b| {
        b.iter(|| r = parallel_solve(&ipl, 6, &res))
    });

    let _ = r;
}

fn resolve_parallel_8(c: &mut Criterion) {
    let res = res_init(ResType::Null);

    let ipl = setup();
    let mut r = IpList::new();

    c.bench_function("parallel_solve 100/8", |b| {
        b.iter(|| r = parallel_solve(&ipl, 8, &res))
    });

    let _ = r;
}

criterion_group!(
    benches,
    resolve_simple,
    resolve_parallel_1,
    resolve_parallel_4,
    resolve_parallel_6,
    resolve_parallel_8
);
criterion_main!(benches);

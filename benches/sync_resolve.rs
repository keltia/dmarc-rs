//! Benchmark for the sync version of the fan-out/fan-in pattern.
//!
//! Please run with `cargo bench`.
//!

use dmarc_rs::res::{res_init, ResType, Solver};

// Std library
//
use std::net::IpAddr;
use std::sync::mpsc::{channel, Receiver};
use std::thread;

// External crates
//
use anyhow::Result;
use criterion::{criterion_group, criterion_main, Criterion};
use itertools::Itertools;
use rayon::prelude::*;
use threadpool::ThreadPool;

/// Number of fake IP in test
///
const MAX_IP: usize = 50;

const RES: ResType = ResType::Sleep;

/// **NOTE** the code is reimplemented here because a bench can not import from the binary, only
/// the library.

/// Simple and straightforward sequential solver
///
/// Example:
/// ```
/// # use dmarc_rs::res::Vec;
/// # use dmarc_rs::res::Ip;
/// let l = Vec::from(["1.1.1.1", "2606:4700:4700::1111", "192.0.2.1"]);
///
/// // select a given resolver
/// let res = res_init(ResType::Null);
///
/// let ptr = simple_solve(l, res);
/// ```
///
fn simple_solve(ipl: &Vec<String>, res: &Solver) -> Vec<Ip> {
    let r: Vec<Ip> = ipl
        .iter()
        .map(|ip| res.solve(&Ip::new(ip)))
        .sorted()
        .collect();
    assert_eq!(ipl.len(), r.len());
    r
}

fn rayon_solve(ipl: &Vec<String>, res: &Solver) -> Vec<Ip> {
    let s = ipl.len();

    let mut r: Vec<Ip> = ipl.par_iter().map(|ip| res.solve(&Ip::new(ip))).collect();

    r.sort();
    assert_eq!(ipl.len(), r.len());
    r
}

fn parallel_solve(ipl: &Vec<String>, workers: usize, res: &Solver) -> Vec<Ip> {
    let s = ipl.len();

    let pool = ThreadPool::new(workers);
    let rx_gen = queue(ipl).unwrap();
    let rx_out = fan_out(rx_gen, pool, s, res).unwrap();
    let full: Vec<Ip> = fan_in(rx_out).unwrap().iter().sorted().collect();

    assert_eq!(ipl.len(), full.len());
    full
}

/// Take all values from the list and send them into a queue
///
fn queue(ipl: &Vec<String>) -> Result<Receiver<String>> {
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
    rx_gen: Receiver<String>,
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
            let name = res.solve(&Ip::new(&n));
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

use dmarc_rs::res::ip::Ip;
use fake::{Fake, Faker};

/// Generate a dataset of N Sleep IpAddr
///
fn setup() -> Vec<String> {
    let mut ipl: Vec<String> = Vec::new();
    for _ in 1..MAX_IP {
        let ip: IpAddr = Faker.fake();
        let s = ip.to_string();
        ipl.push(s.to_owned());
    }
    ipl.clone()
}

fn resolve_simple(c: &mut Criterion) {
    let res = res_init(RES);

    let ipl = setup();
    let mut r = Vec::new();

    c.bench_function("simple_solve 100", |b| {
        b.iter(|| r = simple_solve(&ipl, &res))
    });

    let _ = r;
}

fn resolve_parallel_1(c: &mut Criterion) {
    let res = res_init(RES);

    let ipl = setup();
    let mut r = Vec::new();

    c.bench_function("parallel_solve 100/1", |b| {
        b.iter(|| r = parallel_solve(&ipl, 1, &res))
    });

    let _ = r;
}

fn resolve_parallel_4(c: &mut Criterion) {
    let res = res_init(RES);

    let ipl = setup();
    let mut r = Vec::new();

    c.bench_function("parallel_solve 100/4", |b| {
        b.iter(|| r = parallel_solve(&ipl, 4, &res))
    });

    let _ = r;
}

fn resolve_parallel_6(c: &mut Criterion) {
    let res = res_init(RES);

    let ipl = setup();
    let mut r = Vec::new();

    c.bench_function("parallel_solve 100/6", |b| {
        b.iter(|| r = parallel_solve(&ipl, 6, &res))
    });

    let _ = r;
}

fn resolve_parallel_8(c: &mut Criterion) {
    let res = res_init(RES);

    let ipl = setup();
    let mut r = Vec::new();

    c.bench_function("parallel_solve 100/8", |b| {
        b.iter(|| r = parallel_solve(&ipl, 8, &res))
    });

    let _ = r;
}

fn resolve_rayon(c: &mut Criterion) {
    let res = res_init(RES);

    let ipl = setup();
    let mut r = Vec::new();

    c.bench_function("rayon", |b| b.iter(|| r = rayon_solve(&ipl, &res)));

    let _ = r;
}

criterion_group!(
    benches,
    resolve_simple,
    resolve_parallel_1,
    resolve_parallel_4,
    resolve_parallel_6,
    resolve_parallel_8,
    resolve_rayon,
);
criterion_main!(benches);

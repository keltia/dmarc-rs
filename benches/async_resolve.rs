// Also look in Cargo.toml how to use a benchmark setup with harness = false
// Async version of the parallel_resolve() function.

use dmarc_rs::ip::Ip;
use dmarc_rs::iplist::IpList;
use dmarc_rs::resolver::*;

// Std library
//
use std::net::IpAddr;

// External crates
//
use anyhow::Result;
use async_std::task;
use criterion::async_executor::FuturesExecutor;
use criterion::{criterion_group, criterion_main, Criterion};
use futures::channel::mpsc::{channel, Receiver};
use futures::sink::SinkExt;
use futures::stream::StreamExt;

pub type Error = Box<(dyn std::error::Error + Send + Sync + 'static)>;

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
async fn parallel_solve(ipl: &IpList, _workers: usize, res: &Solver) -> Result<IpList, Error> {
    let mut full = IpList::new();
    let s = ipl.len();

    let mut rx_fan_in = fan_in(fan_out(queue(ipl).await?, s, res).await?).await?;
    loop {
        match rx_fan_in.next().await {
            Some(ip) => full.push(ip),
            None => break,
        }
    }
    //full.sort();
    Ok(full)
}

/// Take all values from the list and send them into a queue
///
async fn queue(ipl: &IpList) -> Result<Receiver<Ip>, Error> {
    let (mut tx, rx) = channel(0);

    // construct a copy of the list
    let all = ipl.clone();

    // use that copy to send over
    task::spawn(async move {
        for ip in all {
            tx.send(ip).await.expect("can not queue")
        }
    });
    Ok(rx)
}

/// Start enough workers to resolve IP into PTR.
///
async fn fan_out(
    mut rx_gen: Receiver<Ip>,
    njobs: usize,
    res: &Solver,
) -> Result<Receiver<Ip>, Error> {
    let (tx, rx) = channel(njobs);

    let mut handles = Vec::new();
    loop {
        match rx_gen.next().await {
            Some(ip) => {
                let mut tx_num = tx.clone();
                let res = res.clone();
                let handle = task::spawn(async move {
                    let name: Ip = res.solve(&ip);
                    tx_num.send(name).await.expect("can not send");
                });
                handles.push(handle);
            }
            _ => break,
        }
    }

    for handle in handles {
        handle.await;
    }
    Ok(rx)
}

/// Gather all results into an output channel
///
async fn fan_in(mut rx_fan_out: Receiver<Ip>) -> Result<Receiver<Ip>> {
    let (mut tx, rx) = channel(0);
    task::spawn(async move {
        loop {
            match rx_fan_out.next().await {
                Some(ip) => tx.send(ip).await.expect("can not send"),
                _ => break,
            }
        }
    });
    Ok(rx)
}

use fake::{Fake, Faker};

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
    let res = res_init(ResType::Null);

    let ipl = setup();
    let mut r = IpList::new();

    c.bench_function("simple_solve 100", |b| {
        b.iter(|| r = simple_solve(&ipl, &res))
    });

    let _ = r;
}

fn resolve_parallel(c: &mut Criterion) {
    let res = res_init(ResType::Null);

    let ipl = setup();
    let r = IpList::new();
    let size: usize = 100;

    c.bench_function("async parallel_solve 100", |b| {
        b.to_async(FuturesExecutor)
            .iter(|| parallel_solve(&ipl, 1, &res))
    });

    let _ = r;
}

criterion_group!(benches, resolve_parallel);
criterion_main!(benches);

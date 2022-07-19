// Also look in Cargo.toml how to use a benchmark setup with harness = false
// Async version of the parallel_resolve() function.

//! We use `IpList` as container and `resolve()` is the main function to get all names.  As we have
//! the choice between two solvers, you can select the simple single-threaded one by specifying
//! that you want only 1 job.
//!
//! When the crate is compiled, the number of CPU & CPU threads is read and that gives us the upper
//! bound for the parallelism.  The `dmarc-cat` binary will default to number physical cores but the hard
//! limit is the number of total core threads (which is higher if the CPU supports Hyperthreading).
//!
//! **NOTE** I have no idea how CPU with different cores types (Apple M1 family or others) are handled,
//! not sure it would make any difference in this case.
//!
//! We define a list of IP tuples from the `dmarc_rs::ip` crate and implement two methods
//! for resolving the IP into names.  One is `simple_solve()` which is a straightforward sequential
//! solver, the other one is `parallel_solve()` which is using threads from a pool to implement a
//! worker-based fan-out/fan-in scheme with channels to move data around.
//!
//! You can select the resolving module to be used from the three defined in `dmarc_rs::resolver`.
//!
//! Examples:
//! ```rust
//! # use dmarc_rs::iplist::IpList;
//! # use dmarc_rs::resolver::*;
//! let l = IpList::from(["1.1.1.1", "2606:4700:4700::1111", "192.0.2.1"]);
//! let res = res_init(ResType::Real);
//!
//! // Use the simple solver
//! let ptr = resolve(&l, 1, &res);
//! dbg!(&ptr);
//! ```
//! and with the parallel solver but with the default resolver:
//! ```rust
//! # use dmarc_rs::iplist::IpList;
//! # use dmarc_rs::resolver::*;
//! // Get the number of physical cores, I prefer to use this one instead of the potentially
//! // larger total cores because Hyperthreading has some overhead.
//! let njobs = num_cpus::get_physical();
//!
//! let l = IpList::from(["1.1.1.1", "2606:4700:4700::1111", "192.0.2.1"]);
//! let res = res_init(ResType::default());
//!
//! // Use the parallel solver
//! let ptr = parallel_solve(&l, njobs, res);
//! dbg!(&ptr);
//! ```
//!

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
pub async fn parallel_solve(ipl: &IpList, _workers: usize, res: &Solver) -> Result<IpList, Error> {
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

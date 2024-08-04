#![feature(isqrt)]
#![feature(try_blocks)]

use std::sync::mpsc::{channel, Receiver};
use std::thread::spawn;
use std::time::SystemTime;

use clap::Parser;
use num_bigint::BigInt;
use rayon::prelude::*;

#[derive(clap::Parser, Debug)]
struct Args {
    #[arg(default_value = "0", short, long)]
    start: u128,
    #[arg(default_value = "340282366920938463463374607431768211455", short, long)]
    end: u128,
    /// Enable benchmark stopwatch timer
    #[arg(short, long)]
    benchmark: bool,
    /// Disable stdout printing
    #[arg(short, long)]
    no_print: bool,
}

fn main() {
    let args = Args::parse();

    let start_time = SystemTime::now();

    let results = run(args.start, args.end);
    for (n, p1, p2) in results {
        let verify = verify(n, p1, p2);
        debug_assert!(verify, "This is expected to be true");
        if !verify {
            continue;
        }
        if !args.no_print {
            println!("{n}: {p1}|{p2}");
        }
    }

    if args.benchmark {
        let duration = SystemTime::now().duration_since(start_time).unwrap();
        println!("Runtime: {:?}", duration);
    }
}

fn run(start: u128, end: u128) -> Receiver<(u128, u128, u128)> {
    let (tx, rx) = channel();

    spawn(move || {
        let tx = tx;
        let base_max = end.isqrt();
        let base_min = start.isqrt();
        let base_range = base_min..=base_max;
        base_range
            .into_par_iter()
            .map(|x| x * x)
            .for_each_with(tx, |s, x| {
                if let Some(r) = check_split(x) {
                    s.send((x, r.0, r.1)).unwrap();
                }
            });
    });
    rx
}

#[inline(always)]
fn check_split(n: u128) -> Option<(u128, u128)> {
    let mut i = 1_u32;
    loop {
        let m = 10_u128.pow(i);
        let part1 = n / m;
        let part2 = n % m;
        if (part1 + part2).checked_mul(part1 + part2) == Some(n) {
            return Some((part1, part2));
        }
        if part1 < 10 {
            break None;
        }
        i += 1;
    }
}

fn verify(n: u128, p1: u128, p2: u128) -> bool {
    (BigInt::from(p1) + BigInt::from(p2)).pow(2) == BigInt::from(n)
}

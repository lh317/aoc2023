use std::fs;

use eyre::{eyre, Result, WrapErr};

fn main() -> Result<()> {
    let mut args = std::env::args();
    let fname = args.nth(1).ok_or(eyre!("filename was not provided"))?;
    let body = fs::read_to_string(fname.clone())?;
    let mut lines = body.lines();
    let (_, times_str) = lines
        .next()
        .and_then(|l| l.split_once(':'))
        .ok_or_else(|| eyre!("{}:1: input error", fname))?;
    let times = times_str
        .split_whitespace()
        .map(|n| n.parse::<u64>().wrap_err(format!("{}:1:", fname)))
        .collect::<Result<Vec<_>>>()?;
    let (_, distance_str) = lines
        .next()
        .and_then(|l| l.split_once(':'))
        .ok_or_else(|| eyre!("{}:2: input error", fname))?;
    let distance = distance_str
        .split_whitespace()
        .map(|n| n.parse::<u64>().wrap_err(format!("{}:1:", fname)))
        .collect::<Result<Vec<_>>>()?;
    let product: usize = times
        .iter()
        .zip(distance.iter())
        .map(|(time, record)| {
            (0..*time)
                .filter(|v| {
                    let travel_time = time - v;
                    let distance = v * travel_time;
                    distance > *record
                })
                .count()
        })
        .product();
    println!("{}", product);
    let single_time = times_str.split_whitespace().collect::<Vec<_>>().join("").parse::<u64>()?;
    let single_distance =
        distance_str.split_whitespace().collect::<Vec<_>>().join("").parse::<u64>()?;
    let single_count = (0..single_time)
        .filter(|v| {
            let travel_time = single_time - v;
            let distance = v * travel_time;
            distance > single_distance
        })
        .count();
    println!("{}", single_count);
    Ok(())
}

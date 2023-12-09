use eyre::{eyre, Report, Result, WrapErr};
use std::fs;

fn main() -> Result<()> {
    let mut args = std::env::args();
    let fname = args.nth(1).ok_or_else(|| eyre!("filename was not provided"))?;
    let body = fs::read_to_string(fname.clone())?;
    let sum: i64 = body
        .lines()
        .enumerate()
        .map(|(lineno, l)| {
            let seq: Vec<_> = l
                .split_whitespace()
                .map(|t| {
                    t.parse::<i64>()
                        .map_err(|_| eyre!("{}:{}: invalid num {}", fname, lineno + 1, t))
                })
                .collect::<Result<_>>()?;
            if seq.len() >= 2 {
                let mut last = Vec::new();
                last.push(*seq.last().unwrap());
                let mut diff: Vec<_> = seq.windows(2).map(|w| w[1] - w[0]).collect();
                loop {
                    last.push(*diff.last().unwrap());
                    if diff.iter().all(|&d| d == diff[0]) {
                        break;
                    }
                    diff = diff.windows(2).map(|w| w[1] - w[0]).collect();
                }
                Ok(last.into_iter().sum::<i64>())
            } else {
                Err(eyre!("{}:{}: too few numbers", fname, lineno +1))
            }
        })
        .sum::<Result<_>>()?;
    println!("{}", sum);
    Ok(())
}

use eyre::{eyre, Result};
use std::fs;

fn main() -> Result<()> {
    let mut args = std::env::args();
    let fname = args.nth(1).ok_or_else(|| eyre!("filename was not provided"))?;
    let body = fs::read_to_string(fname.clone())?;
    let (first_sum, last_sum): (i64, i64) = body
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
                let mut first = Vec::new();
                let mut last = Vec::new();
                first.push(*seq.first().unwrap());
                last.push(*seq.last().unwrap());
                let mut diff: Vec<_> = seq.windows(2).map(|w| w[1] - w[0]).collect();
                loop {
                    first.push(*diff.first().unwrap());
                    last.push(*diff.last().unwrap());
                    if diff.iter().all(|&d| d == diff[0]) {
                        break;
                    }
                    diff = diff.windows(2).map(|w| w[1] - w[0]).collect();
                }
                // Note the order of ops in the rfold: subtract previous diff - accumulated total
                // to go "up" the triangle.
                Ok((first.into_iter().rfold(0, |acc, f| f - acc), last.into_iter().sum::<i64>()))
            } else {
                Err(eyre!("{}:{}: too few numbers", fname, lineno + 1))
            }
        })
        .try_fold((0, 0), |(f0, l0), r| r.map(|(f, l)| (f0 + f, l0 + l)))?;
    println!("{}", last_sum);
    println!("{}", first_sum);
    Ok(())
}

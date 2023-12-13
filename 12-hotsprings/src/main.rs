use std::str::FromStr;

use eyre::{eyre, Report, Result, WrapErr};
use itertools::Itertools;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Spring {
    Working,
    Broken,
    Unknown,
}

impl FromStr for Spring {
    type Err = Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "." => Ok(Spring::Working),
            "#" => Ok(Spring::Broken),
            "?" => Ok(Spring::Unknown),
            _ => Err(eyre!("unknown spring '{s}'")),
        }
    }
}

#[derive(Debug, Clone)]
struct Record {
    springs: Vec<Spring>,
    broken_runs: Vec<usize>,
}

impl Record {
    fn total_broken(&self) -> usize {
        self.broken_runs.iter().sum()
    }

    fn known_broken(&self) -> usize {
        self.springs.iter().enumerate().filter_map(|(i, &s)| if s == Spring::Broken {
            Some(i)
        } else {
            None
        }).count()
    }

    fn unknown_indices(&self) -> impl Iterator<Item=usize> + '_ {
        self.springs.iter().enumerate().filter_map(|(i, &s)| if s == Spring::Unknown {
            Some(i)
        } else {
            None
        })
    }

    fn fill_unknown(&self, broken_indices: impl IntoIterator<Item = usize>) -> Option<Vec<Spring>> {
        let mut result = self.springs.clone();
        for i in broken_indices {
            *result.get_mut(i)? = Spring::Broken;
        }
        for s in &mut result {
            if *s == Spring::Unknown {
                *s = Spring::Working;
            }
        }
        Some(result)
    }

    fn is_solution(&self, candidate: impl IntoIterator<Item = Spring>) -> bool {
        candidate
            .into_iter()
            .dedup_with_count()
            .filter_map(|(n, spring)| {
                if spring == Spring::Broken {
                    Some(n)
                } else {
                    None
                }
            })
            .zip(self.broken_runs.iter())
            .all(|(cn, &en)| cn == en)
    }

    fn grow(&mut self, n: usize) {
        let springs_len = self.springs.len();
        let broken_len = self.broken_runs.len();
        for _ in 0..n {
            self.springs.extend_from_within(0..springs_len);
            self.springs.push(Spring::Unknown);
            self.broken_runs.extend_from_within(0..broken_len);
        }
        self.springs.pop();
    }

}

impl FromStr for Record {
    type Err = Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (spring_str, broken_str) =
            s.split_once(' ').ok_or_else(|| eyre!("invalid record: {}", s))?;
        let springs: Vec<_> =
            spring_str.chars().map(|c| c.to_string().parse::<Spring>()).collect::<Result<_>>()?;
        let broken_runs: Vec<_> = broken_str
            .split(',')
            .map(|s| s.parse::<usize>().wrap_err_with(|| format!("could not parse {}", s)))
            .collect::<Result<_>>()?;
        Ok(Record {
            springs,
            broken_runs,
        })
    }
}

fn main() -> Result<()> {
    let mut args = std::env::args();
    let fname = args.nth(1).ok_or_else(|| eyre!("filename was not provided"))?;
    let body = std::fs::read_to_string(fname.as_str())?;
    let records: Vec<Record> = body
        .lines()
        .enumerate()
        .map(|(lineno, l)| l.parse().wrap_err_with(|| format!("{}:{}", fname, lineno + 1)))
        .collect::<Result<_>>()?;
    let mut sum = 0usize;
    for r in records.iter() {
        let k = r.total_broken() - r.known_broken();
        sum += r.unknown_indices().combinations(k).filter_map(|indices| {
            let filled = r.fill_unknown(indices.into_iter()).ok_or_else(|| eyre!("bad index"));
            match filled {
                Ok(f) => if r.is_solution(f) {
                    Some(Ok(1usize))
                } else {
                    None
                },
                Err(e) => Some(Err(e))
            }
        }).sum::<Result<usize>>()?;
    }
    println!("{sum}");
    Ok(())
}

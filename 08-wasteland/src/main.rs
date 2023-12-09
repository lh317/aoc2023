use std::collections::HashMap;
use std::fs;
use std::str::FromStr;

use eyre::{eyre, Report, Result, WrapErr};
use num::Integer;
use regex::Regex;

#[derive(Debug, Clone, Copy)]
enum Direction {
    Left,
    Right,
}

impl FromStr for Direction {
    type Err = Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "L" => Ok(Direction::Left),
            "R" => Ok(Direction::Right),
            _ => Err(eyre!("unknown direction '{:?}'", s)),
        }
    }
}

fn solve<'a>(
    directions: impl IntoIterator<IntoIter = impl Iterator<Item = &'a Direction> + Clone>,
    map: &HashMap<&str, [&str; 2]>,
    start: &str,
    goals: &[&str],
) -> Result<usize> {
    let mut pos = map.get_key_value(start).ok_or_else(|| eyre!("no key '{}'", start))?;
    directions
        .into_iter()
        .cycle()
        .map_while(|dir| {
            if goals.contains(pos.0) {
                None
            } else {
                let dest = pos.1[*dir as usize];
                Some(match map.get_key_value(dest) {
                    Some(kv) => {
                        pos = kv;
                        Ok(1)
                    }
                    None => Err(eyre!("no key '{}'", dest)),
                })
            }
        })
        .sum()
}

fn main() -> Result<()> {
    let mut args = std::env::args();
    let fname = args.nth(1).ok_or_else(|| eyre!("filename was not provided"))?;
    let body = fs::read_to_string(fname.clone())?;
    let mut lines = body.lines();
    let directions: Vec<_> = {
        let line = lines.next().ok_or_else(|| eyre!("{}:1: unexpected EOF", fname))?;
        line.chars()
            .map(|c| c.to_string().parse::<Direction>())
            .collect::<Result<_>>()
            .wrap_err(format!("{}:1:", fname))?
    };
    let re = Regex::new(r"^(\w+) = \((\w+), (\w+)\)\s*$").unwrap();
    lines.next().ok_or_else(|| eyre!("{}:2: unexpected EOF", fname))?;
    let mut map = HashMap::new();
    for (lineno, line) in (3usize..).zip(lines) {
        let captures =
            re.captures(line).ok_or_else(|| eyre!("{}:{}: does not match regex", fname, lineno))?;
        map.insert(
            captures.get(1).unwrap().as_str(),
            [captures.get(2).unwrap().as_str(), captures.get(3).unwrap().as_str()],
        );
    }
    let steps = solve(directions.iter(), &map, "AAA", &["ZZZ"])?;
    println!("{}", steps);
    let goals: Vec<_> = map.keys().filter(|k| k.ends_with('Z')).copied().collect();
    let steps2: Vec<_> = map
        .keys()
        .filter(|k| k.ends_with('A'))
        .map(|k| solve(directions.as_slice(), &map, k, goals.as_slice()))
        .collect::<Result<_>>()?;
    println!("{:?}", steps2);
    println!("{}", steps2.into_iter().reduce(|acc, s| acc.lcm(&s)).unwrap());
    // let starts = map.keys().filter(|k| k.ends_with('A')).copied();

    // let steps2 = solve(directions.as_slice(), &map, starts, goals.as_slice())?;
    // println!("{}", steps2);
    Ok(())
}

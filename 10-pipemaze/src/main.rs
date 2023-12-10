use std::collections::HashMap;
use std::collections::VecDeque;
use std::str::FromStr;

use eyre::{bail, eyre, Report, Result, WrapErr};
use ndarray::{Array2, ArrayView2};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Direction {
    North,
    South,
    East,
    West
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Pipe {
    NorthSouth,
    EastWest,
    NorthEast,
    NorthWest,
    SouthWest,
    SouthEast,
    Start,
    Ground
}

impl FromStr for Pipe {
    type Err = Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "S" => Ok(Pipe::Start),
            "." => Ok(Pipe::Ground),
            "|" => Ok(Pipe::NorthSouth),
            "-" => Ok(Pipe::EastWest),
            "L" => Ok(Pipe::NorthEast),
            "J" => Ok(Pipe::NorthWest),
            "7" => Ok(Pipe::SouthWest),
            "F" => Ok(Pipe::SouthEast),
            _ => Err(eyre!("unknown pipe '{}'", s))
        }
    }
}

fn connected_start(map: ArrayView2<Pipe>, start: [usize; 2]) -> Option<[[usize; 2]; 2]> {
    let [row, col] = start;
    let above = map.get([row.saturating_sub(1), col]);
    let below = map.get([row +1, col]);
    let left = map.get([row, col.saturating_sub(1)]);
    let right = map.get([row, col+1]);
    let mut solve = Vec::new();
    match above {
        Some(Pipe::SouthWest)|Some(Pipe::NorthSouth)|Some(Pipe::SouthEast) => solve.push(Direction::North),
        _ => (),
    };
    match below {
        Some(Pipe::NorthSouth)|Some(Pipe::NorthEast)|Some(Pipe::NorthWest) => solve.push(Direction::South),
        _ => (),
    }
    match left {
        Some(Pipe::EastWest)|Some(Pipe::NorthEast)|Some(Pipe::SouthEast) => solve.push(Direction::West),
        _ => (),
    }
    match right {
        Some(Pipe::EastWest)|Some(Pipe::NorthWest)|Some(Pipe::SouthWest) => solve.push(Direction::East),
        _ => ()
    }
    if solve.len() == 2 {
        match (solve[0], solve[1]) {
            (Direction::North, Direction::South) => Some([[row-1, col], [row+1, col]]),
            (Direction::North, Direction::West) => Some([[row-1, col], [row, col-1]]),
            (Direction::North, Direction::East) => Some([[row-1, col], [row, col+1]]),
            (Direction::South, Direction::West) => Some([[row+1, col], [row, col-1]]),
            (Direction::South, Direction::East) => Some([[row+1, col], [row, col+1]]),
            _ => None
        }
    } else {
        None
    }
}

fn main() -> Result<()> {
    let mut args = std::env::args();
    let fname = args.nth(1).ok_or_else(|| eyre!("filename was not provided"))?;
    let body = std::fs::read_to_string(fname.as_str())?;
    let mut columns = 0usize;
    let mut rows = 0usize;
    let mut start = [0usize, 0usize];
    let mut values = Vec::new();
    for line in body.lines() {
        rows += 1;
        if rows == 1 {
            columns = line.len();
        }
        if line.len() == columns {
            for (col, c) in line.chars().enumerate() {
                let p = c.to_string().parse::<Pipe>().wrap_err_with(|| format!("{}:{}", fname, rows))?;
                values.push(p);
                if p == Pipe::Start {
                    start = [rows - 1, col];
                }
            }
        } else {
            bail!("{}:{}: expected {} columns got {}", fname, rows, columns, line.len());
        }
    }
    let map = Array2::from_shape_vec((rows, columns), values)?;
    let mut edges = HashMap::from([(start, 0)]);
    let mut stack = VecDeque::from([(0usize, start)]);
    while let Some((d, [row, col])) = stack.pop_front() {
        let next = match map.get([row, col]) {
            None => bail!("invalid index: ({}, {})", row, col),
            Some(&Pipe::Ground) => bail!("index ({},{}) is ground!", row, col),
            Some(&Pipe::NorthSouth) => [[row.saturating_sub(1), col], [row+1, col]],
            Some(&Pipe::EastWest) => [[row, col.saturating_sub(1)], [row, col+1]],
            Some(&Pipe::NorthEast) => [[row.saturating_sub(1), col], [row, col+1]],
            Some(&Pipe::NorthWest) => [[row.saturating_sub(1), col], [row, col.saturating_sub(1)]],
            Some(&Pipe::SouthWest) => [[row+1, col], [row, col.saturating_sub(1)]],
            Some(&Pipe::SouthEast) => [[row+1, col], [row, col + 1]],
            Some(Pipe::Start) => connected_start(map.view(), [row, col]).ok_or_else(|| eyre!("invalid start ({}, {})", row, col))?,
        };
        for pos in next.into_iter() {
            edges.entry(pos).or_insert_with(|| {
                stack.push_back((d+1, pos));
                d + 1
            });
        }
    }
    println!("{}", edges.values().max().unwrap());
    Ok(())
}

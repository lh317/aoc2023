use std::collections::HashSet;
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

impl Pipe {
    fn is_up(&self) -> bool {
        matches!(self, Pipe::NorthSouth | Pipe::NorthEast | Pipe::NorthWest)
    }
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

fn from_start(map: ArrayView2<Pipe>, start: [usize; 2]) -> Option<Pipe> {
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
            (Direction::North, Direction::South) => Some(Pipe::NorthSouth),
            (Direction::North, Direction::West) => Some(Pipe::NorthWest),
            (Direction::North, Direction::East) => Some(Pipe::NorthEast),
            (Direction::South, Direction::West) => Some(Pipe::SouthWest),
            (Direction::South, Direction::East) => Some(Pipe::SouthEast),
            (Direction::West, Direction::East) => Some(Pipe::EastWest),
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
    let mut map = Array2::from_shape_vec((rows, columns), values)?;
    map[start] = from_start(map.view(), start).ok_or_else(|| eyre!("invalid start"))?;
    let mut edges = HashSet::from([start]);
    let mut stack = VecDeque::from([start]);
    while let Some([row, col]) = stack.pop_front() {
        let next = match map.get([row, col]) {
            None => bail!("invalid index: ({}, {})", row, col),
            Some(&Pipe::Ground) => bail!("index ({},{}) is ground!", row, col),
            Some(&Pipe::NorthSouth) => [[row.saturating_sub(1), col], [row+1, col]],
            Some(&Pipe::EastWest) => [[row, col.saturating_sub(1)], [row, col+1]],
            Some(&Pipe::NorthEast) => [[row.saturating_sub(1), col], [row, col+1]],
            Some(&Pipe::NorthWest) => [[row.saturating_sub(1), col], [row, col.saturating_sub(1)]],
            Some(&Pipe::SouthWest) => [[row+1, col], [row, col.saturating_sub(1)]],
            Some(&Pipe::SouthEast) => [[row+1, col], [row, col + 1]],
            _ => bail!("index ({}, {}) is still start!", row, col),
        };
        for pos in next.into_iter() {
            if !edges.contains(&pos) {
                stack.push_back(pos);
                edges.insert(pos);
            };
        }
    }
    println!("{}", edges.len() / 2);
    let mut inside = 0usize;
    // A point is inside a closed shape if a ray in any direction crosses an odd
    // number of times.
    // Trick: Need to only count up or down, not both, when casting left -> right.
    for row in 0..rows {
        for col in 0..columns {
            if !edges.contains(&[row, col]) {
                let crossings = (0..col).filter(|&c| match edges.get(&[row, c]) {
                    Some(pos) => map[*pos].is_up(),
                    None => false
                }).count();
                if crossings & 1 == 1 {
                    inside += 1;
                }
            }
        }
    }
    println!("{}", inside);
    Ok(())
}

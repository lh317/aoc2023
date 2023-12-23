use std::collections::{HashMap, HashSet};
use std::str::FromStr;

use eyre::{eyre, Report, Result, OptionExt, WrapErr};
use itertools::Itertools;
use ndarray::{Array2, s, DataMut, ArrayBase, Ix2};
use rgb::RGB8;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Direction {
    Up,
    Down,
    Left,
    Right
}

impl FromStr for Direction {
    type Err = Report;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "U" => Ok(Direction::Up),
            "D" => Ok(Direction::Down),
            "L" => Ok(Direction::Left),
            "R" => Ok(Direction::Right),
            _ => Err(eyre!("invalid direction '{value}'"))
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct Dig {
    dir: Direction,
    steps: u8,
    color: RGB8
}

impl FromStr for Dig {
    type Err = Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut tokens = s.split_whitespace();
        let dir_str = tokens.next().ok_or_eyre("missing direction")?;
        let steps_str = tokens.next().ok_or_eyre("missing steps")?;
        let color_str = tokens.next().ok_or_eyre("missing color")?;
        let dir = dir_str.parse()?;
        let steps = steps_str.parse()?;
        let color_hex = hex::decode(color_str.strip_prefix("(#").and_then(|s| s.strip_suffix(')')).ok_or_eyre("invalid color format '{color_str}'")?)?;
        let color = RGB8::from_iter(color_hex);
        Ok(Dig { dir, steps, color })
    }
}

fn flood_fill<T: Default, D: DataMut<Elem=Option<T>>>(
    array: &mut ArrayBase<D, Ix2>,
    edges: &HashMap<usize, Vec<usize>>,
) {
    let (rows, cols) = array.dim();
    for row in 0..rows {
        for col in 0..cols {
            if array[[row, col]].is_none() {
                let count = edges.get(&row).map_or(0, |r| r.iter().filter(|&&c| c > col).count());
                if count & 1 == 1 {
                    array[[row, col]] = Some(Default::default());
                }
            }
        }
    }
}

fn main() -> Result<()> {
    let mut args = std::env::args();
    let fname = args.nth(1).ok_or_eyre("filename was not provided")?;
    let body = std::fs::read_to_string(fname.as_str())?;
    let digs: Vec<Dig> = body.lines().enumerate().map(|(lineno, l)| l.parse().wrap_err_with(|| format!("{}:{}", fname, lineno+1))).collect::<Result<_>>()?;
    let extents = digs.iter().fold([0usize; 4], |mut acc, d| {acc[d.dir as usize] += d.steps as usize; acc});
    let rows = extents[2] + extents[3];
    let cols = extents[0] + extents[1];
    let mut pos = [extents[0], extents[2]];
    let mut array = Array2::<Option<RGB8>>::default((rows, cols));
    let mut edges = HashMap::new();
    for (i, dig) in digs.iter().enumerate() {
        let end = match dig.dir {
            Direction::Up => [pos[0]- usize::from(dig.steps), pos[1]],
            Direction::Down => [pos[0] + usize::from(dig.steps), pos[1]],
            Direction::Left => [pos[0], pos[1] - usize::from(dig.steps)],
            Direction::Right => [pos[0], pos[1] + usize::from(dig.steps)],
        };
        let slice = match dig.dir {
            Direction::Up => array.slice_mut(s![end[0]..pos[0], pos[1]]),
            Direction::Down => array.slice_mut(s![pos[0]..=end[0], pos[1]]),
            Direction::Left => array.slice_mut(s![pos[0], end[1]..pos[1]]),
            Direction::Right => array.slice_mut(s![pos[0], pos[1]..=end[1]]),
        };
        for x in slice {
            *x = Some(dig.color);
        }
        let range = match dig.dir {
            Direction::Up => end[0]+1..pos[0],
            Direction::Down => pos[0]+1..end[0],
            _ => 0..0,
        };
        for row in range {
            edges.entry(row).or_insert(Vec::new()).push(pos[1]);
        }
        match (dig.dir, digs[(i + 1) % digs.len()].dir) {
            (Direction::Left|Direction::Right, Direction::Up) => edges.entry(pos[0]).or_insert(Vec::new()).push(end[1]),
            (Direction::Down, Direction::Left|Direction::Right) => edges.entry(end[0]).or_insert(Vec::new()).push(pos[1]),
            _ => (),
        };
        pos = end;
    }
    flood_fill(&mut array, &edges);
    let count: usize = array.iter().filter(|x| x.is_some()).count();
    println!("{count}");
    // for row in 0..rows {
    //     for col in 0..cols {
    //         print!("{}", if array[[row, col]].is_some() { "#" } else {"."});
    //     }
    //     print!("\n")
    // }
    Ok(())
}

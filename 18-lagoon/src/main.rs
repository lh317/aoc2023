use std::collections::HashMap;
use std::str::FromStr;

use eyre::{eyre, Report, Result, OptionExt, WrapErr};
use ndarray::{Array2, s, DataMut, ArrayBase, Ix2};
use rgb::{RGB,RGB8};

trait Arrow {
    fn dir(&self) -> Direction;
    fn steps(&self) -> usize;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Direction {
    Right,
    Down,
    Left,
    Up
}

impl From<usize> for Direction {
    fn from(value: usize) -> Self {
        match value {
            0 => Direction::Right,
            1 => Direction::Down,
            2 => Direction::Left,
            3 => Direction::Up,
            _ => panic!("not a valid direction"),
        }
    }
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

impl Arrow for Dig {
    fn dir(&self) -> Direction { self.dir }
    fn steps(&self) -> usize { usize::from(self.steps) }
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

#[derive(Debug, Clone, Copy)]
struct BigDig {
    dir: Direction,
    steps: usize,
}

impl Arrow for BigDig {
    fn dir(&self) -> Direction { self.dir}
    fn steps(&self) -> usize {self.steps}
}

impl From<Dig> for BigDig {
    fn from(value: Dig) -> Self {
        let color: RGB<usize> = RGB::new(value.color.r.into(), value.color.g.into(), value.color.b.into());
        let steps = (color.r << 12) + (color.g << 4) + (color.b >> 4);
        let dir = (color.b & 0x0F).into();
        BigDig {dir, steps}
    }
}


fn solve<A: Arrow>(digs: &[A]) -> usize {
    let mut pos = [0isize; 2];
    let mut vertices = vec![pos];
    let mut perimeter = 0usize;
    for dig in digs.iter() {
        perimeter += dig.steps();
        let steps = isize::try_from(dig.steps()).unwrap();
        pos = match dig.dir() {
            Direction::Up => [pos[0] - steps, pos[1]],
            Direction::Down => [pos[0] + steps, pos[1]],
            Direction::Left => [pos[0], pos[1] - steps],
            Direction::Right => [pos[0], pos[1] + steps],
        };
        vertices.push(pos);
    }
    let mut area = 0.0f64;
    for (v1, v2) in vertices.iter().zip(vertices.iter().cycle().skip(1)) {
        let v1_x: f64 = v1[1] as f64;
        let v1_y: f64 = v1[0] as f64;
        let v2_x: f64 = v2[1] as f64;
        let v2_y: f64 = v2[0] as f64;
        area += v1_x * v2_y;
        area -= v1_y * v2_x;
    }
    area /= 2.0;
    (area + (perimeter as f64 / 2.0)) as usize + 1
}



fn main() -> Result<()> {
    let mut args = std::env::args();
    let fname = args.nth(1).ok_or_eyre("filename was not provided")?;
    let body = std::fs::read_to_string(fname.as_str())?;
    let digs: Vec<Dig> = body.lines().enumerate().map(|(lineno, l)| l.parse().wrap_err_with(|| format!("{}:{}", fname, lineno+1))).collect::<Result<_>>()?;
    let extents = digs.iter().fold([0usize; 4], |mut acc, d| {acc[d.dir as usize] += d.steps as usize; acc});
    let rows = extents[0] + extents[2];
    let cols = extents[1] + extents[3];
    let mut pos = [extents[3], extents[2]];
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
            edges.entry(row).or_insert_with(Vec::new).push(pos[1]);
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
    println!("{}", solve(&digs));
    let bigdigs: Vec<BigDig> = digs.into_iter().map(BigDig::from).collect();
    println!("{}", solve(&bigdigs));
    Ok(())
}

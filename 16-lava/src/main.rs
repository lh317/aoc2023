use std::collections::HashSet;

use eyre::{bail, eyre, Report, Result, WrapErr};
use ndarray::{s, Array2, ArrayBase, Data, Ix2};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Entry {
    Empty,
    MirrorLeftUp,
    MirrorLeftDown,
    SplitHoriz,
    SplitVert,
}

impl TryFrom<char> for Entry {
    type Error = Report;

    fn try_from(c: char) -> std::prelude::v1::Result<Self, Self::Error> {
        match c {
            '.' => Ok(Entry::Empty),
            '/' => Ok(Entry::MirrorLeftUp),
            '\\' => Ok(Entry::MirrorLeftDown),
            '-' => Ok(Entry::SplitHoriz),
            '|' => Ok(Entry::SplitVert),
            _ => Err(eyre!("unknown entry '{c}'")),
        }
    }
}

fn parse_array<'a, I: Iterator<Item = &'a str>>(lines: I) -> Result<Array2<Entry>> {
    let mut rows = 0usize;
    let mut columns = 0usize;
    let mut values = Vec::new();
    for (lineno, line) in lines.enumerate() {
        rows += 1;
        if rows == 1 {
            columns = line.len();
        }
        if line.len() == columns {
            for c in line.chars() {
                values.push(c.try_into()?);
            }
        } else {
            bail!("{}: expected {columns} columns but got {}", lineno + 1, line.len())
        }
    }
    Array2::from_shape_vec((rows, columns), values).wrap_err("converting to array")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

fn occupy<D: Data<Elem = Entry>>(
    array: &ArrayBase<D, Ix2>,
    start: [isize; 2],
    dir: Direction,
) -> usize {
    let mut occupied = Array2::<bool>::default(array.dim());
    let mut cast = HashSet::new();
    let mut rays = vec![(start, dir)];
    while let Some((ray, dir)) = rays.pop() {
        cast.insert((ray, dir));
        let s = match dir {
            Direction::Right => s![ray[0], ray[1] + 1..],
            Direction::Down => s![ray[0] + 1.., ray[1]],
            Direction::Left => s![ray[0], ..ray[1];-1],
            Direction::Up => s![..ray[0];-1, ray[1]],
        };
        let entry_slice = array.slice(s);
        let mut occupied_slice = occupied.slice_mut(s);
        for (offset, (entry, occ)) in entry_slice.iter().zip(occupied_slice.iter_mut()).enumerate()
        {
            let offset = offset + 1;
            *occ = true;
            let offsets = [
                ([0, offset as isize], Direction::Up),
                ([0, offset as isize], Direction::Down),
                ([0, -(offset as isize)], Direction::Up),
                ([0, -(offset as isize)], Direction::Down),
                ([offset as isize, 0], Direction::Left),
                ([offset as isize, 0], Direction::Right),
                ([-(offset as isize), 0], Direction::Left),
                ([-(offset as isize), 0], Direction::Right),
            ];
            let next = match (dir, entry) {
                (Direction::Right, Entry::MirrorLeftUp) => &offsets[0..1],
                (Direction::Right, Entry::MirrorLeftDown) => &offsets[1..2],
                (Direction::Right, Entry::SplitVert) => &offsets[0..2],
                (Direction::Left, Entry::MirrorLeftUp) => &offsets[3..4],
                (Direction::Left, Entry::MirrorLeftDown) => &offsets[2..3],
                (Direction::Left, Entry::SplitVert) => &offsets[2..4],
                (Direction::Down, Entry::MirrorLeftUp) => &offsets[4..5],
                (Direction::Down, Entry::MirrorLeftDown) => &offsets[5..6],
                (Direction::Down, Entry::SplitHoriz) => &offsets[4..6],
                (Direction::Up, Entry::MirrorLeftUp) => &offsets[7..8],
                (Direction::Up, Entry::MirrorLeftDown) => &offsets[6..7],
                (Direction::Up, Entry::SplitHoriz) => &offsets[6..8],
                _ => &[][..],
            };
            if !next.is_empty() {
                for &(no, nd) in next {
                    let pos = [ray[0] + no[0], ray[1] + no[1]];
                    if !cast.contains(&(pos, nd)) {
                        //println!("pushing {:?}", (pos, nd));
                        rays.push((pos, nd));
                    }
                }
                break;
            }
        }
    }
    occupied.into_iter().filter(|&e| e).count()
}

fn main() -> Result<()> {
    let mut args = std::env::args();
    let fname = args.nth(1).ok_or_else(|| eyre!("filename was not provided"))?;
    let body = std::fs::read_to_string(fname.as_str())?;
    let wall = parse_array(body.lines())?;
    println!("{}", occupy(&wall, [0, -1], Direction::Right));
    let (rows, cols) = wall.dim();
    let rows_iter = (0..rows as isize).flat_map(|r| {
        [
            occupy(&wall, [r, -1], Direction::Right),
            occupy(&wall, [r, cols as isize], Direction::Left),
        ]
    });
    let cols_iter = (0..cols as isize).flat_map(|c| {
        [occupy(&wall, [-1, c], Direction::Down), occupy(&wall, [rows as isize, c], Direction::Up)]
    });
    let max = rows_iter.chain(cols_iter).max().unwrap();
    println!("{max}");
    Ok(())
}

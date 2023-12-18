#![allow(clippy::mut_range_bound)]
use eyre::{bail, eyre, Report, Result, WrapErr};
use indexmap::{IndexMap, map::Entry};
use ndarray::{s, ArrayBase, Array2, DataMut, Ix2};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Rock {
    Round,
    Cube,
    Empty
}

impl TryFrom<char> for Rock {
    type Error = Report;

    fn try_from(c: char) -> Result<Self, Self::Error> {
        match c {
            '.' => Ok(Rock::Empty),
            '#' => Ok(Rock::Cube),
            'O' => Ok(Rock::Round),
            _ => Err(eyre!("unknown rock '{c}'"))
        }
    }
}

fn parse_array<'a, I: Iterator<Item=&'a str>>(lines: I) -> Result<Array2<Rock>> {
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

fn cycle<D: DataMut<Elem=Rock>>(array: &mut ArrayBase<D, Ix2>) {
    let (rows, cols) = array.dim();
    // North
    for mut col in array.columns_mut() {
        let mut start = col.iter().position(|&r| r == Rock::Empty).unwrap_or(cols);
        let mut end = col.slice(s![start..]).iter().position(|&r| r == Rock::Cube).unwrap_or(cols - start) + start;
        while start < cols {
            for i in start..end {
                if col[i] == Rock::Round {
                    col[start] = Rock::Round;
                    col[i] = Rock::Empty;
                    start += 1;
                }
            }
            start = col.slice(s![end..]).iter().position(|&r| r == Rock::Empty).unwrap_or(cols - end) + end;
            end = col.slice(s![start..]).iter().position(|&r| r == Rock::Cube).unwrap_or(cols - start) + start;
        }
    }
    // West
    for mut row in array.rows_mut() {
        let mut start = row.iter().position(|&r| r == Rock::Empty).unwrap_or(rows);
        let mut end = row.slice(s![start..]).iter().position(|&r| r == Rock::Cube).unwrap_or(cols - start) + start;
        while start < rows {
            for i in start..end {
                if row[i] == Rock::Round {
                    row[start] = Rock::Round;
                    row[i] = Rock::Empty;
                    start += 1;
                }
            }
            start = row.slice(s![end..]).iter().position(|&r| r == Rock::Empty).unwrap_or(rows - end) + end;
            end = row.slice(s![start..]).iter().position(|&r| r == Rock::Cube).unwrap_or(rows - start) + start;
        }
    }
    // South
    for mut col in array.columns_mut() {
        let mut start = cols - 1 - col.iter().rev().position(|&r| r == Rock::Empty).unwrap_or(cols - 1);
        while start > 0 {
            let end = start - 1 - col.slice(s![..start;-1]).iter().position(|&r| r == Rock::Cube).unwrap_or(start - 1);
            for i in (end..start).rev() {
                if col[i] == Rock::Round {
                    col[start] = Rock::Round;
                    col[i] = Rock::Empty;
                    start -= 1;
                }
            }
            if end > 0 {
                start = end - 1 - col.slice(s![..end;-1]).iter().position(|&r| r == Rock::Empty).unwrap_or(end - 1);
            } else {
                start = 0;
            }
        }
    }
    // East
    for mut row in array.rows_mut() {
        let mut start = rows - 1 - row.iter().rev().position(|&r| r == Rock::Empty).unwrap_or(rows - 1);
        while start > 0 {
            let end = start - 1 - row.slice(s![..start;-1]).iter().position(|&r| r == Rock::Cube).unwrap_or(start -1);
            for i in (end..start).rev() {
                if row[i] == Rock::Round {
                    row[start] = Rock::Round;
                    row[i] = Rock::Empty;
                    start -= 1;
                }
            }
            if end > 0 {
                start = end - 1 - row.slice(s![..end;-1]).iter().position(|&r| r == Rock::Empty).unwrap_or(end - 1);
            } else {
                start = 0;
            }
        }
    }
}

fn main() -> Result<()> {
    let mut args = std::env::args();
    let fname = args.nth(1).ok_or_else(|| eyre!("filename was not provided"))?;
    let body = std::fs::read_to_string(fname.as_str())?;
    let mut array = parse_array(body.lines())?;
    let mut part2 = array.clone();
    let (rows, cols) = array.dim();
    for mut col in array.columns_mut() {
        let mut start = col.iter().position(|&r| r == Rock::Empty).unwrap_or(cols);
        let mut end = col.slice(s![start..]).iter().position(|&r| r == Rock::Cube).unwrap_or(cols - start) + start;
        while start < cols {
            for i in start..end {
                if col[i] == Rock::Round {
                    col[start] = Rock::Round;
                    col[i] = Rock::Empty;
                    start += 1;
                }
            }
            start = col.slice(s![end..]).iter().position(|&r| r == Rock::Empty).unwrap_or(cols - end) + end;
            end = col.slice(s![start..]).iter().position(|&r| r == Rock::Cube).unwrap_or(cols - start) + start;
        }
    }
    let sum: usize = array.rows().into_iter().enumerate().map(|(i, r)| (rows - i) * r.iter().filter(|&&r| r == Rock::Round).count()).sum();
    println!("{sum}");
    let mut map = IndexMap::new();
    let mut index = 0;
    for _ in 1..=1_000_000_000 {
        cycle(&mut part2);
        let l = map.len();
        match map.entry(part2.clone()) {
            Entry::Occupied(e) => {
                println!("Found: {} {l}", e.index());
                let whole = (1_000_000_000 - l - 1) / (l - e.index());
                let mod1 = (1_000_000_000 - l - 1) % (l - e.index());
                println!("whole: {whole}\tmod1: {mod1}");
                println!("{}, {}, {}", whole % (l - e.index()), mod1 % (l - e.index()), (whole + mod1) % (l - e.index()));
                index = mod1 + e.index();
                break;
            },
            Entry::Vacant(e) => e.insert(()),
        };
    }
    //let x = map[index];
    let sum2: usize = map.get_index(index).unwrap().0.rows().into_iter().enumerate().map(|(i, r)| (rows - i) * r.iter().filter(|&&r| r == Rock::Round).count()).sum();
    for (k, _) in map {
        println!("{}", k.rows().into_iter().enumerate().map(|(i, r)| (rows - i) * r.iter().filter(|&&r| r == Rock::Round).count()).sum::<usize>());
    }
    println!("Sum: {sum2} index: {index}");
    Ok(())

}

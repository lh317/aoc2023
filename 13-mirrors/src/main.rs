#![allow(clippy::iter_nth_zero)]

use eyre::{eyre, Report, Result, WrapErr, OptionExt};
use ndarray::{Array2, ArrayBase, Axis, Data, Ix2};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Terrain {
    Ash,
    Rocks,
}

impl Terrain {
    fn flip(&mut self) {
        *self = match self {
            Terrain::Ash => Terrain::Rocks,
            Terrain::Rocks => Terrain::Ash
        };
    }
}
impl TryFrom<char> for Terrain {
    type Error = Report;

    fn try_from(c: char) -> Result<Self, Self::Error> {
        match c {
            '.' => Ok(Terrain::Ash),
            '#' => Ok(Terrain::Rocks),
            _ => Err(eyre!("unknown terrain '{c}'")),
        }
    }
}

fn parse_array<'a>(lines: impl Iterator<Item = &'a str>) -> Option<Result<Array2<Terrain>>> {
    let mut rows = 0usize;
    let mut columns = 0usize;
    let mut values = Vec::<Terrain>::new();
    for line in lines.take_while(|l| !l.is_empty()) {
        rows += 1;
        if rows == 1 {
            columns = line.len();
        }
        if line.len() == columns {
            for c in line.chars() {
                match c.try_into() {
                    Ok(t) => values.push(t),
                    Err(e) => return Some(Err(e)),
                };
            }
        }
    }
    if rows > 0 {
        Some(Array2::from_shape_vec((rows, columns), values).wrap_err("converting to array"))
    } else {
        None
    }
}

fn mirror<'a, T: PartialEq + 'a, D: Data<Elem = T>>(array: &'a ArrayBase<D, Ix2>) -> impl Iterator<Item=(Axis, usize)> + '_ {

    let col_iter = array.columns().into_iter();
    let col_iter_s1 = col_iter.clone().skip(1);
    let col_combined = col_iter.zip(col_iter_s1).enumerate().filter_map(|(i, (c0, c1))| {
        let (_, cols) = array.dim();
        if c0 == c1 && (0..i).rev().zip((i+2)..cols).all(|(lc, rc)| array.column(lc) == array.column(rc)) {
            Some((Axis(1), i))
        } else {
            None
        }
    });
    let row_iter = array.rows().into_iter();
    let row_iter_s1 = row_iter.clone().skip(1);
    let row_combined = row_iter.zip(row_iter_s1).enumerate().filter_map(|(i, (r0, r1))| {
        let (rows, _) = array.dim();
        if r0 == r1 && (0..i).rev().zip((i + 2)..rows).all(|(lr, rr)| array.row(lr) == array.row(rr)) {
            Some((Axis(0), i))
        } else {
            None
        }
    });
    col_combined.chain(row_combined)
}


fn main() -> Result<()> {
    let mut args = std::env::args();
    let fname = args.nth(1).ok_or_else(|| eyre!("filename was not provided"))?;
    let body = std::fs::read_to_string(fname.as_str())?;
    let mut lines = body.lines();
    let mut arrays = Vec::new();
    while let Some(array) = parse_array(lines.by_ref()) {
        arrays.push(array?);
    }
    let mut sum = 0usize;
    let mut results = Vec::new();
    for array in arrays.iter_mut() {
        let answer = mirror(array).nth(0).ok_or_eyre("did not find mirror")?;
        results.push(answer);
        sum += match answer {
            (Axis(0), x) => 100 * (x + 1),
            (Axis(1), y) => y + 1,
            _ => unreachable!(),
        };
    }
    println!("{sum}");
    let mut sum2 = 0usize;
    'array: for (array, answer) in arrays.iter_mut().zip(results.iter()) {
        let (rows, cols) = array.dim();
        for row in 0..rows {
            for col in 0..cols {
                array[[row, col]].flip();
                if let Some(new) = mirror(array).filter(|a| a != answer).nth(0) {
                    sum2 += match new {
                        (Axis(0), x) => 100 * (x + 1),
                        (Axis(1), y) => y+1,
                        _ => unreachable!()
                    };
                    continue 'array;
                }
                array[[row, col]].flip();
            }
        }
        // sum2 += match *answer {
        //     (Axis(0), x) => 100 * (x + 1),
        //     (Axis(1), y) => y+1,
        //     _ => unreachable!()
        // };
    }
    println!("{sum2}");
    Ok(())
}

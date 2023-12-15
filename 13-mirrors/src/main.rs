use eyre::{eyre, Report, Result, WrapErr};
use ndarray::Array2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Terrain {
    Ash,
    Rocks,
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
    for array in arrays.iter() {
        let (rows, cols) = array.dim();
        for col in array
            .columns()
            .into_iter()
            .zip(array.columns().into_iter().skip(1))
            .enumerate()
            .filter_map(|(i, (col, next_col))| {
                if col == next_col {
                    Some(i)
                } else {
                    None
                }
            })
        {
            if (0..col).rev().zip(col + 2..cols).all(|(c, mc)| array.column(c) == array.column(mc))
            {
                sum += col + 1;
            }
        }

        for row in
            array.rows().into_iter().zip(array.rows().into_iter().skip(1)).enumerate().filter_map(
                |(i, (row, next_row))| {
                    if row == next_row {
                        Some(i)
                    } else {
                        None
                    }
                },
            )
        {
            if (0..row).rev().zip(row + 2..rows).all(|(r, mr)| array.row(r) == array.row(mr)) {
                sum += 100 * (row + 1);
            }
        }
    }
    println!("{sum}");
    Ok(())
}

use eyre::{bail, eyre, Report, Result, WrapErr};
use ndarray::{s, Array2};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
    Ok(())

}

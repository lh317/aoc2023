use eyre::{bail, eyre, Result};
use itertools::Itertools;
use ndarray::Array2;

fn main() -> Result<()> {
    let mut args = std::env::args();
    let fname = args.nth(1).ok_or_else(|| eyre!("filename was not provided"))?;
    let body = std::fs::read_to_string(fname.as_str())?;
    let mut columns = 0usize;
    let mut rows = 0usize;
    let mut values = Vec::new();
    for line in body.lines() {
        rows += 1;
        if rows == 1 {
            columns = line.len();
        }
        if line.len() == columns {
            for c in line.chars() {
                match c {
                    '#' => values.push(true),
                    '.' => values.push(false),
                    _ => bail!("{}:{}: unexpected char '{}'", fname, rows, c),
                };
            }
        } else {
            bail!("{}:{}: expected {} columns got {}", fname, rows, columns, line.len());
        }
    }
    let universe = Array2::from_shape_vec((rows, columns), values)?;
    let empty_cols: Vec<_> = universe
        .columns()
        .into_iter()
        .enumerate()
        .filter_map(|(i, r)| {
            if r.into_iter().all(|g| !g) {
                Some(i)
            } else {
                None
            }
        })
        .collect();
    let empty_rows: Vec<_> = universe
        .rows()
        .into_iter()
        .enumerate()
        .filter_map(|(i, r)| {
            if r.into_iter().all(|g| !g) {
                Some(i)
            } else {
                None
            }
        })
        .collect();
    let galaxies: Vec<_> = universe
        .indexed_iter()
        .filter_map(|((r0, c0), g)| {
            if *g {
                let r = empty_rows.iter().take_while(|r| **r < r0).count() + r0;
                let c = empty_cols.iter().take_while(|c| **c < c0).count() + c0;
                Some((r, c))
            } else {
                None
            }
        })
        .collect();
    let mut sum = 0isize;
    for combo in galaxies.iter().combinations(2) {
        let &(g1x, g1y) = combo[0];
        let &(g2x, g2y) = combo[1];
        let d = ((g1x as isize) - (g2x as isize)).abs() + ((g1y as isize) - (g2y as isize)).abs();
        //println!("({}, {}) -> ({}, {}): {}", g1x, g1y, g2x, g2y, d);
        sum += d;
    }
    println!("{}", sum);
    let galaxies2: Vec<_> = universe
        .indexed_iter()
        .filter_map(|((r0, c0), g)| {
            if *g {
                let r = empty_rows.iter().take_while(|r| **r < r0).count() * 999_999 + r0;
                let c = empty_cols.iter().take_while(|c| **c < c0).count() * 999_999 + c0;
                Some((r, c))
            } else {
                None
            }
        })
        .collect();
    let mut sum2 = 0isize;
    for combo in galaxies2.iter().combinations(2) {
        let &(g1x, g1y) = combo[0];
        let &(g2x, g2y) = combo[1];
        let d = ((g1x as isize) - (g2x as isize)).abs() + ((g1y as isize) - (g2y as isize)).abs();
        //println!("({}, {}) -> ({}, {}): {}", g1x, g1y, g2x, g2y, d);
        sum2 += d;
    }
    println!("{}", sum2);
    Ok(())
}

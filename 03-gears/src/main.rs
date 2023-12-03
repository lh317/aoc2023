use eyre::{eyre, Result};
use ndarray::{s, Array2};
use std::fs;

fn main() -> Result<()> {
    let mut args = std::env::args();
    let fname = args.nth(1).ok_or(eyre!("filename was not provided"))?;
    let body = fs::read_to_string(fname.clone())?;
    let mut columns = 0usize;
    let mut rows = 0usize;
    let mut values = Vec::new();
    for line in body.lines() {
        rows += 1;
        for c in line.chars() {
            values.push(c)
        }
        if rows == 1 {
            columns = values.len();
        }
    }
    let schematic = Array2::from_shape_vec((rows, columns), values)?;
    let mut sum = 0u32;
    for ((row, col), c) in schematic.indexed_iter() {
        if !c.is_ascii_digit() && *c != '.' {
            let left_count = schematic
                .slice(s!(row,..col;-1))
                .into_iter()
                .take_while(|c| c.is_ascii_digit())
                .count();
            let left_num: String =
                schematic.slice(s!(row, (col - left_count)..col)).into_iter().collect();
            if !left_num.is_empty() {
                sum += left_num.parse::<u32>()?;
            }
            if col < columns - 1 {
                let right_num: String = schematic
                    .slice(s!(row, (col + 1)..))
                    .into_iter()
                    .take_while(|c| c.is_ascii_digit())
                    .collect();
                if !right_num.is_empty() {
                    sum += right_num.parse::<u32>()?;
                }
            }
            if row > 0 {
                // top
                let mut topleft_num = if col > 0 {
                    let count = schematic
                        .slice(s!(row -1,..col;-1))
                        .into_iter()
                        .take_while(|c| c.is_ascii_digit())
                        .count();
                    schematic.slice(s!(row - 1, (col - count)..col)).into_iter().collect()
                } else {
                    String::new()
                };
                let top_num: String = schematic
                    .slice(s!(row - 1, col..col + 1))
                    .into_iter()
                    .take_while(|c| c.is_ascii_digit())
                    .collect();
                let topright_num = if col < columns - 1 {
                    schematic
                        .slice(s!(row - 1, (col + 1)..))
                        .into_iter()
                        .take_while(|c| c.is_ascii_digit())
                        .collect()
                } else {
                    String::new()
                };
                if !top_num.is_empty() {
                    topleft_num.push_str(top_num.as_str());
                    topleft_num.push_str(topright_num.as_str());
                } else if !topright_num.is_empty() {
                        sum += topright_num.parse::<u32>()?;
                }
                if !topleft_num.is_empty() {
                    sum += topleft_num.parse::<u32>()?;
                }
            }
            if row < rows - 1 {
                // bottom
                let mut topleft_num = if col > 0 {
                    let count = schematic
                        .slice(s!(row + 1,..col;-1))
                        .into_iter()
                        .take_while(|c| c.is_ascii_digit())
                        .count();
                    schematic.slice(s!(row + 1, (col - count)..col)).into_iter().collect()
                } else {
                    String::new()
                };
                let top_num: String = schematic
                    .slice(s!(row + 1, col..col + 1))
                    .into_iter()
                    .take_while(|c| c.is_ascii_digit())
                    .collect();
                let topright_num = if col < columns - 1 {
                    schematic
                        .slice(s!(row + 1, (col + 1)..))
                        .into_iter()
                        .take_while(|c| c.is_ascii_digit())
                        .collect()
                } else {
                    String::new()
                };
                if !top_num.is_empty() {
                    topleft_num.push_str(top_num.as_str());
                    topleft_num.push_str(topright_num.as_str());
                } else if !topright_num.is_empty() {
                        sum += topright_num.parse::<u32>()?;
                }
                if !topleft_num.is_empty() {
                    sum += topleft_num.parse::<u32>()?;
                }
            }
        }
    }
    println!("{}", sum);
    Ok(())
}

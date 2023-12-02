use std::fs;
use std::io;

use eyre::{eyre};

fn main() -> eyre::Result<()> {
    let mut args = std::env::args();
    let fname = args.nth(1).ok_or(eyre!("filename was not provided"))?;
    let body = fs::read_to_string(fname.clone())?;
    let mut sum = 0i32;

    'line: for (lineno, line) in body.lines().enumerate() {
        let (id_str, shows) = line.split_once(":").ok_or_else(|| eyre!("{}:{}: invalid line", fname, lineno+1))?;
        let id: i32 = id_str[5..].parse()?;
        for show in shows.split(";") {
            for dice in show.split(",") {
                let (num_str, color) = dice.trim().split_once(" ").ok_or(eyre!("{}: {}: invalid line", line, lineno + 1))?;
                let num: i32 = num_str.parse().map_err(|_| eyre!("{}: {}: could not parse '{}'", fname, lineno + 1, num_str))?;
                match color {
                    "red" if num > 12 => continue 'line,
                    "green" if num > 13 => continue 'line,
                    "blue" if num > 14 => continue 'line,
                    _ => (),
                };
            }
        }
        sum += id;
    }
    println!("{}", sum);

    Ok(())
}

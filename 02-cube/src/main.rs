use std::fs;
use std::io;

use eyre::eyre;

#[derive(Debug, Clone, Copy)]
struct Rolls {
    red: Option<i32>,
    green: Option<i32>,
    blue: Option<i32>
}

impl Rolls {
    fn new() -> Self {
        Self { red: None, green: None, blue: None }
    }

    fn update(self, color: &str, num: i32) -> eyre::Result<Self> {
        match color {
            "red" => Ok(Self {red: Some(num), green: self.green, blue: self.blue}),
            "green" => Ok(Self {red: self.red, green: Some(num), blue: self.blue}),
            "blue" => Ok(Self {red: self.red, green: self.green, blue: Some(num)}),
            _ => Err(eyre!("unknown color {}", color)),
        }
    }

    fn keep_max_color(&mut self, rhs: &Self) {
        self.red = match (self.red, rhs.red) {
            (Some(lhs), Some(rhs)) => Some(std::cmp::max(lhs, rhs)),
            (None, Some(rhs)) => Some(rhs),
            (Some(lhs), None) => Some(lhs),
            _ => None
        };
        self.green = match (self.green, rhs.green) {
            (Some(lhs), Some(rhs)) => Some(std::cmp::max(lhs, rhs)),
            (None, Some(rhs)) => Some(rhs),
            (Some(lhs), None) => Some(lhs),
            _ => None
        };
        self.blue = match (self.blue, rhs.blue) {
            (Some(lhs), Some(rhs)) => Some(std::cmp::max(lhs, rhs)),
            (None, Some(rhs)) => Some(rhs),
            (Some(lhs), None) => Some(lhs),
            _ => None
        };
    }

    fn power(&self) -> i32 { self.red.unwrap_or(0) * self.green.unwrap_or(0) * self.blue.unwrap_or(0) }

    fn valid(&self) -> bool { self.red.unwrap_or(0) <= 12 && self.green.unwrap_or(0) <= 13 && self.blue.unwrap_or(0) <= 14 }
}

fn main() -> eyre::Result<()> {
    let mut args = std::env::args();
    let fname = args.nth(1).ok_or(eyre!("filename was not provided"))?;
    let body = fs::read_to_string(fname.clone())?;
    let mut sum = 0i32;
    let mut power_sum = 0i32;
    for (lineno, line) in body.lines().enumerate() {
        let mut max_rolls = Rolls::new();
        let mut valid = true;
        let (id_str, shows) = line.split_once(":").ok_or_else(|| eyre!("{}:{}: invalid line", fname, lineno+1))?;
        let id: i32 = id_str[5..].parse()?;
        for show in shows.split(";") {
            let mut rolls = Rolls::new();
            for dice in show.split(",") {
                let (num_str, color) = dice.trim().split_once(" ").ok_or(eyre!("{}: {}: invalid line", line, lineno + 1))?;
                let num: i32 = num_str.parse().map_err(|_| eyre!("{}: {}: could not parse '{}'", fname, lineno + 1, num_str))?;
                rolls = rolls.update(color, num)?;
            }
            max_rolls.keep_max_color(&rolls);
            valid = valid && rolls.valid();
        }
        if valid {
            sum += id;
        }
        power_sum += max_rolls.power();
    }
    println!("{}", sum);
    println!("{}", power_sum);
    Ok(())
}

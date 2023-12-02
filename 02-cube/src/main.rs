use std::fs;

use eyre::eyre;

#[derive(Debug, Clone, Copy, Default)]
struct Rolls {
    red: i32,
    green: i32,
    blue: i32,
}

impl Rolls {
    fn new(red: i32, green: i32, blue: i32) -> Self {
        Self {
            red,
            green,
            blue,
        }
    }

    fn keep_max_color(&mut self, rhs: &Self) {
        self.red = std::cmp::max(self.red, rhs.red);
        self.green = std::cmp::max(self.green, rhs.green);
        self.blue = std::cmp::max(self.blue, rhs.blue);
    }

    fn power(&self) -> i32 {
        self.red * self.green * self.blue
    }

    fn valid(&self) -> bool {
        self.red <= 12 && self.green <= 13 && self.blue <= 14
    }
}

#[derive(Debug, Clone, Copy, Default)]
struct RollsBuilder {
    red: Option<i32>,
    green: Option<i32>,
    blue: Option<i32>,
}

impl RollsBuilder {
    fn new() -> Self {
        Self::default()
    }

    fn add_color(&mut self, color: &str, num: i32) -> eyre::Result<()> {
        match color {
            "red" => {
                self.red = Some(num);
                Ok(())
            }
            "green" => {
                self.green = Some(num);
                Ok(())
            }
            "blue" => {
                self.blue = Some(num);
                Ok(())
            }
            _ => Err(eyre!("unknown color {}", color)),
        }
    }

    fn build(self) -> Rolls {
        Rolls::new(self.red.unwrap_or(0), self.green.unwrap_or(0), self.blue.unwrap_or(0))
    }
}

fn main() -> eyre::Result<()> {
    let mut args = std::env::args();
    let fname = args.nth(1).ok_or(eyre!("filename was not provided"))?;
    let body = fs::read_to_string(fname.clone())?;
    let mut sum = 0i32;
    let mut power_sum = 0i32;
    for (lineno, line) in body.lines().enumerate() {
        let mut max_rolls = Rolls::default();
        let mut valid = true;
        let (id_str, shows) =
            line.split_once(':').ok_or_else(|| eyre!("{}:{}: invalid line", fname, lineno + 1))?;
        let id: i32 = id_str[5..].parse()?;
        for show in shows.split(';') {
            let mut builder = RollsBuilder::new();
            for dice in show.split(',') {
                let (num_str, color) = dice.trim().split_once(' ').ok_or(eyre!(
                    "{}: {}: invalid line",
                    line,
                    lineno + 1
                ))?;
                let num: i32 = num_str.parse().map_err(|_| {
                    eyre!("{}: {}: could not parse '{}'", fname, lineno + 1, num_str)
                })?;
                builder.add_color(color, num)?;
            }
            let rolls = builder.build();
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

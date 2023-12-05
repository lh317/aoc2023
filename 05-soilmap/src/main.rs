use std::fs;
use std::str::FromStr;

use eyre::{eyre, Report, Result, WrapErr};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Map {
    source: u64,
    dest: u64,
    length: u64,
}
impl Map {
    fn source_end(&self) -> u64 { self.source + self.length }

    fn source_contains(&self, index: u64) -> bool {
        index >= self.source && index < self.source_end()
    }
}

impl FromStr for Map {
    type Err = eyre::Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let tokens = s
            .split_whitespace()
            .map(|t| t.parse().wrap_err("parsing integer"))
            .collect::<Result<Vec<u64>>>()?;
        if tokens.len() == 3 {
            Ok(Map {
                source: tokens[1],
                dest: tokens[0],
                length: tokens[2],
            })
        } else {
            Err(eyre!("expected 3 numbers on line got {}", tokens.len()))
        }
    }
}

fn read_map<'a>(lines: &mut impl Iterator<Item = &'a str>) -> Result<Vec<Map>> {
    let mut result = lines
        .take_while(|l| !l.is_empty())
        .map(|l| l.parse::<Map>())
        .collect::<Result<Vec<_>>>()?;
    result.sort();
    Ok(result)
}

fn lookup(table: &[Map], source: u64) -> u64 {
    match table.binary_search_by_key(&source, |m| m.source) {
        Ok(index) => table[index].dest,
        Err(index) => {
            if index > 0 {
                let map = table[index-1];
                if map.source_contains(source) {
                    map.dest + (source - map.source)
                } else {
                    source
                }
            } else {
                source
            }
        }
    }
}

fn main() -> Result<()> {
    let mut args = std::env::args();
    let fname = args.nth(1).ok_or(eyre!("filename was not provided"))?;
    let body = fs::read_to_string(fname.clone())?;
    let mut lines = body.lines();
    let seeds = {
        let seed_line = lines.next().ok_or(eyre!("{}:1: unexpected EOF", fname))?;
        let (_, seed_tokens) =
            seed_line.split_once(':').ok_or(eyre!("{}:1: missing seeds", fname))?;
        seed_tokens
            .split_whitespace()
            .map(|t| t.parse().wrap_err("integer"))
            .collect::<Result<Vec<u64>>>()?
    };
    lines.next().ok_or(eyre!("{}:2: unexpected EOF", fname))?;
    // seed-to-soil map:
    lines.next().ok_or(eyre!("{}:3: unexpected EOF", fname))?;
    let seed_soil = read_map(lines.by_ref())?;
    // soil-to-fertilizer map:
    lines.next().ok_or(eyre!("{}:soil-fert: unexpected EOF", fname))?;
    let soil_fertilizer = read_map(lines.by_ref())?;
    // fertilizer-to-water map:
    lines.next().ok_or(eyre!("{}:fert-water: unexpected EOF", fname))?;
    let fertilizer_water = read_map(lines.by_ref())?;
    // water-to-light map:
    lines.next().ok_or(eyre!("{}:water-light: unexpected EOF", fname))?;
    let water_light = read_map(lines.by_ref())?;
    // light-temp map:
    lines.next().ok_or(eyre!("{}:light-temp: unexpected EOF", fname))?;
    let light_temp = read_map(lines.by_ref())?;
    // temp-humidity map:
    lines.next().ok_or(eyre!("{}:temp-humidity: unexpected EOF", fname))?;
    let temp_humidity = read_map(lines.by_ref())?;
    // humidity-location map:
    lines.next().ok_or(eyre!("{}:humidity-location: unexpected EOF", fname))?;
    let humidity_location = read_map(lines.by_ref())?;
    let min_location = seeds.iter().map(|s| {
        let soil = lookup(&seed_soil, *s);
        let fert = lookup(&soil_fertilizer, soil);
        let water = lookup(&fertilizer_water, fert);
        let light = lookup(&water_light, water);
        let temp = lookup(&light_temp, light);
        let humidity = lookup(&temp_humidity, temp);
        lookup(&humidity_location, humidity)
    }).min().ok_or(eyre!("minimum not found"))?;
    println!("{}", min_location);
    Ok(())
}

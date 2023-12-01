use std::fs;
use std::io;

use aho_corasick::AhoCorasick;

fn main() -> Result<(), io::Error> {
    let mut args = std::env::args();
    let fname = args.nth(1).expect("filename was not provided");
    let body = fs::read_to_string(fname.clone())?;
    let mut sum = 0i32;
    for (lineno, line) in body.lines().enumerate() {
        let mut digits = String::new();
        for c in line.chars() {
            if c.is_ascii_digit() {
                digits.push(c);
            }
        }
        if digits.len() == 1 {
            digits.push(digits.chars().next().unwrap());
        }
        if digits.len() > 2 {
            digits.replace_range(1..digits.len() - 1, "");
        }
        sum += digits
            .parse::<i32>()
            .unwrap_or_else(|_| panic!("{}:{}: number not found", fname, lineno + 1));
    }
    println!("{}", sum);
    sum = 0;
    let patterns = &[
        "one", "two", "three", "four", "five", "six", "seven", "eight", "nine", "1", "2", "3", "4",
        "5", "6", "7", "8", "9",
    ];
    let digits = &[1, 2, 3, 4, 5, 6, 7, 8, 9, 1, 2, 3, 4, 5, 6, 7, 8, 9];
    let ac = AhoCorasick::new(patterns).unwrap();
    for line in body.lines() {
        let nums: Vec<_> =
            ac.find_overlapping_iter(line).map(|m| digits[m.pattern().as_usize()]).collect();
        let value = nums.first().unwrap() * 10 + nums.last().unwrap();
        sum += value;
    }
    println!("{}", sum);
    Ok(())
}

use std::hash::{Hasher, Hash};

use eyre::{eyre, Result};

#[derive(Debug, Clone, Copy, Default)]
struct HolidayHasher {
    value: u64
}

impl HolidayHasher {
    fn new() -> Self {
        Self::default()
    }
}

impl Hasher for HolidayHasher {
    fn finish(&self) -> u64 {
        self.value
    }

    fn write(&mut self, bytes: &[u8]) {
        for &b in bytes {
            self.value += b as u64;
            self.value *= 17;
            self.value %= 256;
        }
    }
}

fn main() -> Result<()> {
    let mut args = std::env::args();
    let fname = args.nth(1).ok_or_else(|| eyre!("filename was not provided"))?;
    let body = std::fs::read_to_string(fname.as_str())?;
    let sum: u64 = body.trim_end().split(',').map(|s| {
        let mut hasher = HolidayHasher::new();
        hasher.write(s.as_bytes());
        let result = hasher.finish();
        //println!("{s}: {result}");
        result
    }).sum();
    println!("{sum}");
    Ok(())
}

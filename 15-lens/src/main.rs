use std::hash::Hasher;
use std::str::FromStr;

use eyre::{eyre, Report, Result, OptionExt};
use indexmap::IndexMap;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum InstructionKind {
    Remove,
    Add(u8)
}

impl FromStr for InstructionKind {
    type Err = Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "-" {
            Ok(InstructionKind::Remove)
        } else if s.starts_with('=') {
            Ok(InstructionKind::Add(s.strip_prefix('=').unwrap().parse()?))
        } else {
            Err(eyre!("unknown instruction '{s}'"))
        }
    }
}


#[derive(Debug, Clone, Copy)]
struct Instruction<'source> {
    label: &'source str,
    op: InstructionKind,
}

impl<'a> Instruction<'a> {
    fn label(&self) -> &'a str {
        self.label
    }

    fn op(&self) -> &InstructionKind {
        &self.op
    }

    fn light_box(&self) -> usize {
        let mut hasher = HolidayHasher::new();
        hasher.write(self.label.as_bytes());
        hasher.finish() as usize
    }
}

impl<'source> TryFrom<&'source str> for Instruction<'source> {
    type Error = Report;

    fn try_from(s: &'source str) -> Result<Self, Self::Error> {
        let index = s.find(['=', '-']).ok_or_eyre("invalid instruction '{s}'")?;
        let (label, op_str) = s.split_at(index);
        let op = op_str.parse()?;
        Ok(Self {label, op })
    }
}


fn main() -> Result<()> {
    let mut args = std::env::args();
    let fname = args.nth(1).ok_or_else(|| eyre!("filename was not provided"))?;
    let body = std::fs::read_to_string(fname.as_str())?;
    let sum: u64 = body.trim_end().split(',').map(|s| {
        let mut hasher = HolidayHasher::new();
        hasher.write(s.as_bytes());
        hasher.finish()
    }).sum();
    println!("{sum}");
    let mut boxes: [IndexMap<&str, u8>; 256] = array_init::array_init(|_| IndexMap::new());
    for s in body.trim_end().split(',') {
        let inst: Instruction = s.try_into()?;
        let b = &mut boxes[inst.light_box()];
        match inst.op() {
            InstructionKind::Remove => {b.shift_remove(inst.label());},
            InstructionKind::Add(lens) => {*b.entry(inst.label()).or_default() = *lens;},
        };
    }
    let sum2: usize = boxes.iter().enumerate().flat_map(|(i, b)| {
        b.iter().enumerate().map(move |(j, (_, lens))| (i+1)*(j+1)*(*lens as usize))
    }).sum();
    println!("{sum2}");
    Ok(())
}

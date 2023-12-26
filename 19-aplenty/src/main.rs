use std::collections::HashMap;
use std::iter::FusedIterator;
use std::ops::Range;
use std::str::FromStr;

use byteyarn::{yarn, Yarn};
use eyre::{eyre, OptionExt, Report, Result, Context};

#[derive(Debug, Clone, Copy)]
enum Rating {
    Extreme,
    Musical,
    Aerodynamic,
    Shiny,
}

impl TryFrom<char> for Rating {
    type Error = Report;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            'x' => Ok(Rating::Extreme),
            'm' => Ok(Rating::Musical),
            'a' => Ok(Rating::Aerodynamic),
            's' => Ok(Rating::Shiny),
            _ => Err(eyre!("invalid category '{value}'")),
        }
    }
}

#[derive(Debug, Clone)]
enum Destination {
    Accept,
    Reject,
    Rule(Yarn),
}

impl FromStr for Destination {
    type Err = Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "A" => Ok(Destination::Accept),
            "R" => Ok(Destination::Reject),
            _ => Ok(Destination::Rule(Yarn::copy(s))),
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum Operation {
    LessThan,
    GreaterThan,
}

impl TryFrom<char> for Operation {
    type Error = Report;

    fn try_from(c: char) -> Result<Self, Self::Error> {
        match c {
            '<' => Ok(Operation::LessThan),
            '>' => Ok(Operation::GreaterThan),
            _ => Err(eyre!("bad operand '{c}'")),
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct Comparison {
    rating: Rating,
    op: Operation,
    value: u32,
}

impl Comparison {
    fn call(&self, part: &Part) -> bool {
        let lhs = match self.rating {
            Rating::Extreme => part.extreme,
            Rating::Musical => part.musical,
            Rating::Aerodynamic => part.aerodynamic,
            Rating::Shiny => part.shiny
        };
        match self.op {
            Operation::LessThan => lhs < self.value,
            Operation::GreaterThan => lhs > self.value
        }
    }

    fn split_constraint(&self, pc: &PartConstraint) -> (PartConstraint, PartConstraint) {
        let range = match self.rating {
            Rating::Extreme => &pc.extreme,
            Rating::Musical => &pc.musical,
            Rating::Aerodynamic => &pc.aerodynamic,
            Rating::Shiny => &pc.shiny
        };
        let offset = match self.op {
            Operation::LessThan => 0,
            Operation::GreaterThan => 1,
        };
        let less_than = pc.clone().update_range(self.rating, range.start..(self.value+offset));
        let greater_than = pc.clone().update_range(self.rating, (self.value+offset)..range.end);
        match self.op {
            Operation::LessThan => (less_than, greater_than),
            Operation::GreaterThan => (greater_than, less_than),
        }
    }

}

impl FromStr for Comparison {
    type Err = Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut chars = s.chars();
        let kind_str = chars.next().ok_or_else(|| eyre!("comparsion missing category: {s}"))?;
        let rating = kind_str.try_into()?;
        let op_str = chars.next().ok_or_else(|| eyre!("comparsion missing operator: {s}"))?;
        let op = op_str.try_into()?;
        let value = chars.as_str().parse()?;
        Ok(Comparison {
            rating,
            op,
            value,
        })
    }
}

#[derive(Debug, Clone)]
struct Rule {
    comparsion: Option<Comparison>,
    destination: Destination,
}

impl Rule {
    fn process(&self, part: &Part) -> Option<&Destination> {
        if let Some(comp) = self.comparsion {
            if comp.call(part) {
                Some(&self.destination)
            } else {
                None
            }
        } else {
            Some(&self.destination)
        }
    }

    fn constrain(&self, pc: &PartConstraint) -> (PartConstraint, &Destination, Option<PartConstraint>) {
        if let Some(comp) = self.comparsion {
            let (branch, cont) = comp.split_constraint(pc);
            (branch, &self.destination, Some(cont))
        } else {
            (pc.clone(), &self.destination, None)
        }
    }
}

impl FromStr for Rule {
    type Err = Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let split = s.split_once(':');
        if let Some((comparsion_str, dest_str)) = split {
            let comparsion = Some(comparsion_str.parse()?);
            let destination = dest_str.parse()?;
            Ok(Rule {
                comparsion,
                destination,
            })
        } else {
            let destination = s.parse()?;
            Ok(Rule {
                comparsion: None,
                destination,
            })
        }
    }
}

#[derive(Debug, Clone)]
struct Flow(Vec<Rule>);

impl Flow {
    fn process(&self, part: &Part) -> Option<&Destination> {
        self.0.iter().flat_map(|r| r.process(part)).next()
    }

    fn constrain<'this: 'iter, 'iter>(&'this self, pc: PartConstraint) -> impl Iterator<Item=(PartConstraint, &'this Destination)> + 'iter {
        let mut current = Some(pc);
        self.0.iter().filter_map(move |r| if let Some(pc) = &current {
            let (branch, dest, next) = r.constrain(pc);
            current = next;
            Some((branch, dest))
        } else { None }).fuse()
    }
}

impl FromStr for Flow {
    type Err = Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let v = s.split(',').map(|s| s.parse()).collect::<Result<_>>()?;
        Ok(Flow(v))
    }
}

#[derive(Debug, Clone)]
struct NamedFlow(Yarn, Flow);

impl NamedFlow {
    fn into_tuple(self) -> (Yarn, Flow) {
        (self.0, self.1)
    }
}

impl FromStr for NamedFlow {
    type Err = Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (name_str, flow_str) = s.split_once('{').ok_or_eyre("named flow missing '{'")?;
        let name = Yarn::copy(name_str);
        let flow = flow_str.trim_end_matches('}').parse()?;
        Ok(NamedFlow(name, flow))
    }
}

#[derive(Debug, Clone, Copy)]
struct Part {
    extreme: u32,
    musical: u32,
    aerodynamic: u32,
    shiny: u32,
}

impl Part {
    fn is_accepted(&self, map: &HashMap<Yarn, Flow>) -> Result<bool> {
        let mut key = &yarn!("in");
        loop {
            let flow = map.get(key).ok_or_else(|| eyre!("invalid flow '{key}"))?;
            key = match flow.process(self) {
                None => return Err(eyre!("flow '{key}' did not terminate: {flow:?}")),
                Some(Destination::Accept) => return Ok(true),
                Some(Destination::Reject) => return Ok(false),
                Some(Destination::Rule(next)) => next,
            };
        }
    }

    fn total_rating(&self) -> u32 {
        self.extreme + self.musical + self.aerodynamic + self.shiny
    }
}

impl FromStr for Part {
    type Err = Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let trimmed = s.trim_start_matches('{').trim_end_matches('}');
        let mut values: [Option<u32>; 4] = [None; 4];
        for (i, part_str) in trimmed.split(',').enumerate() {
            if i >= 4 {
                return Err(eyre!("too many terms in '{s}'"));
            }
            values[i] = Some(part_str.split_once('=').ok_or_else(|| eyre!("bad term '{part_str}'")).and_then(|(_, n)| n.parse().wrap_err("bad integer"))?);
        }
        if values.iter().all(|v| v.is_some()) {
            Ok(Part {
                extreme: values[0].unwrap(),
                musical: values[1].unwrap(),
                aerodynamic: values[2].unwrap(),
                shiny: values[3].unwrap()
            })
        } else {
            Err(eyre!("too few terms in '{s}'"))
        }
    }
}


#[derive(Debug, Clone)]
struct PartConstraint {
    extreme: Range<u32>,
    musical: Range<u32>,
    aerodynamic: Range<u32>,
    shiny: Range<u32>,
}

impl PartConstraint {
    fn total_rating(&self) -> u64 {
        [&self.extreme, &self.musical, &self.aerodynamic, &self.shiny].iter().map(|r| u64::from(r.end - r.start)).product()
    }

    fn update_range(self, rating: Rating, new: Range<u32>) -> Self {
        match rating {
            Rating::Extreme => PartConstraint { extreme: new, ..self },
            Rating::Musical => PartConstraint { musical: new, ..self },
            Rating::Aerodynamic => PartConstraint { aerodynamic: new, ..self},
            Rating::Shiny => PartConstraint { shiny: new, ..self},
        }
    }
}

#[derive(Debug, Clone)]
struct ConstraintSolver<'map> {
    map: &'map HashMap<Yarn, Flow>,
    stack: Option<Vec<(PartConstraint, &'map Destination)>>
}

impl<'map> Iterator for ConstraintSolver<'map> {
    type Item = Result<PartConstraint>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.stack.is_none() {
            let start = PartConstraint {
                extreme: 1..4001,
                musical: 1..4001,
                aerodynamic: 1..4001,
                shiny: 1..4001,
            };
            let flow = match self.map.get("in") {
                Some(f) => f,
                None => return Some(Err(eyre!("invalid flow 'in'"))),
            };
            self.stack = Some(Vec::from_iter(flow.constrain(start)));
        }
        let stack = self.stack.as_mut().unwrap();
        while let Some(pc) = stack.pop() {
            let key = match pc.1 {
                Destination::Reject => continue,
                Destination::Accept => return Some(Ok(pc.0)),
                Destination::Rule(dest) => dest,
            };
            let flow = match self.map.get(key) {
                Some(f) => f,
                None => return Some(Err(eyre!("invalid flow '{key}'"))),
            };
            stack.extend(flow.constrain(pc.0));
        }
        None
    }
}

impl FusedIterator for ConstraintSolver<'_> {}

fn constraint_solver(map: &HashMap<Yarn, Flow>) -> ConstraintSolver {
    ConstraintSolver {
        map,
        stack: None,
    }
}


fn main() -> Result<()> {
    let mut args = std::env::args();
    let fname = args.nth(1).ok_or_eyre("filename was not provided")?;
    let body = std::fs::read_to_string(fname.as_str())?;
    let mut lines = body.lines();
    let map: HashMap<Yarn, Flow> = lines.by_ref()
        .take_while(|l| !l.is_empty())
        .map(|l| l.parse::<NamedFlow>().map(NamedFlow::into_tuple))
        .collect::<Result<_>>()?;
    let parts: Vec<Part> = lines.map(|l| l.parse()).collect::<Result<_>>()?;
    let total: u32 = parts.iter().filter_map(|p| match p.is_accepted(&map) {
        Ok(true) => Some(Ok(p.total_rating())),
        Ok(false) => None,
        Err(e) => Some(Err(e)),
    }).sum::<Result<_>>()?;
    println!("{total}");
    let combos: u64 = constraint_solver(&map).map(|r| r.map(|pc| {
        pc.total_rating()
    })).sum::<Result<_>>()?;
    println!("{combos}");
    Ok(())
}


use enum_iterator::{self, Sequence};
use eyre::{bail, eyre, Result, WrapErr, OptionExt};
use itertools::Itertools;
use ndarray::{Array2, ArrayBase, Data, Ix2, RawData};
use petgraph::{Directed, Graph};
use petgraph::algo::astar;

fn parse_array<'a, I: Iterator<Item = &'a str>>(lines: I) -> Result<Array2<usize>> {
    let mut rows = 0usize;
    let mut columns = 0usize;
    let mut values = Vec::new();
    for (lineno, line) in lines.enumerate() {
        rows += 1;
        if rows == 1 {
            columns = line.len();
        }
        if line.len() == columns {
            for c in line.chars() {
                values.push(c.to_digit(10).ok_or_eyre("invalid digit '{c}'")? as usize);
            }
        } else {
            bail!("{}: expected {columns} columns but got {}", lineno + 1, line.len())
        }
    }
    Array2::from_shape_vec((rows, columns), values).wrap_err("converting to array")
}

// Matches slice formation in edges
#[derive(Debug, Clone, Copy, PartialEq, Eq, Sequence)]
enum Direction {
    Right,
    Left,
    Down,
    Up
}

impl From<usize> for Direction {
    fn from(value: usize) -> Self {
        match value {
            0 => Direction::Right,
            1 => Direction::Left,
            2 => Direction::Down,
            3 => Direction::Up,
            _ => panic!("bad index"),
        }
    }
}

#[derive(Debug)]
struct Node(isize, isize, Direction, usize);

impl Node {
    fn from_id(id: usize, cols: usize) -> Node {
        let steps = id % 4;
        let dir = ((id / 4) % 4).into();
        let base = id / 16;
        let row = base / cols;
        let col = base % cols;
        Node (row as isize, col as isize, dir, steps)
    }

    fn id(&self, rows: usize, cols: usize) -> usize {
        self.0 as usize * cols * 16 + self.1 as usize * 16  + self.2 as usize * 4 + self.3
    }

    fn index(&self) -> [usize; 2] {
        [self.0 as usize, self.1 as usize]
    }

    fn in_bound(&self, rows: usize, cols: usize) -> bool {
        let rows = rows as isize;
        let cols = cols as isize;
        self.0 >= 0 && self.0 < rows && self.1 >= 0 && self.1 < cols
    }

    fn left(&self) -> [Option<Node>; 3] {
        let mut result = [None, None, None];
        let dir = self.2;
        if dir != Direction::Right {
            let stop = 4 - if dir == Direction::Left { self.3 } else {0};
            for (step, r) in (1..stop).zip(result.iter_mut()) {
                *r = Some(Node(self.0, self.1 - (step as isize), Direction::Left, step));
            }
        }
        result
    }

    fn right(&self) -> [Option<Node>; 3] {
        let mut result = [None, None, None];
        let dir = self.2;
        if dir != Direction::Left {
            let step_off = if dir == Direction::Right { self.3 } else { 0 };
            let stop = 4 - step_off;
            for (step, r) in (1..stop).zip(result.iter_mut()) {
                *r = Some(Node(self.0, self.1 + (step as isize), Direction::Right, step + step_off));
            }
        }
        result
    }

    fn up(&self) -> [Option<Node>; 3] {
        let mut result = [None, None, None];
        let dir = self.2;
        if dir != Direction::Down {
            let step_off = if dir == Direction::Up { self.3 } else {0};
            let stop = 4 - step_off;
            for (step, r) in (1..stop).zip(result.iter_mut()) {
                *r = Some(Node(self.0 - (step as isize), self.1, Direction::Up, step + step_off));
            }
        }
        result
    }

    fn down(&self) -> [Option<Node>; 3] {
        let mut result = [None, None, None];
        let dir = self.2;
        if dir != Direction::Up {
            let step_off = if dir == Direction::Down { self.3 } else {0};
            let stop = 4 - step_off;
            for (step, r) in (1..stop).zip(result.iter_mut()) {
                *r = Some(Node(self.0 + (step as isize), self.1, Direction::Down, step + step_off));
            }
        }
        result
    }

    fn edges<D: RawData>(self, array: &ArrayBase<D, Ix2>) -> Edges<ArrayBase<D, Ix2>> {
        let items = self.right();
        Edges {
            node: self,
            array,
            total: 0,
            dir: Direction::Right,
            items,
            index: 0
        }
    }

}


struct Edges<'array, A> {
    node: Node,
    array: &'array A,
    total: usize,
    dir: Direction,
    items: [Option<Node>; 3],
    index: usize
}

impl<'array, D: Data<Elem=usize>> Iterator for Edges<'array, ArrayBase<D, Ix2>> {
    type Item = (usize, usize, usize);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.index > 2 {
                (self.dir, self.items) = match self.dir {
                    Direction::Right => (Direction::Down, self.node.down()),
                    Direction::Down => (Direction::Left, self.node.left()),
                    Direction::Left => (Direction::Up, self.node.up()),
                    Direction::Up => return None,
                };
                self.index = 0;
                self.total = 0;
            }
            let index = self.index;
            self.index += 1;
            if let Some(node) = &self.items[index] {
                let (rows, cols) = self.array.dim();
                if node.in_bound(rows, cols) {
                    self.total += self.array[node.index()];
                    let x = Some((self.node.id(rows, cols), node.id(rows, cols), self.total));
                    return x;
                }
            }
        }
    }
}


fn all_edges<D: Data<Elem=usize>>(array: &ArrayBase<D, Ix2>) -> impl Iterator<Item=(usize, usize, usize)> + '_ {
    array.indexed_iter().flat_map(move |((row, col), _)| {
        enum_iterator::all::<Direction>().cartesian_product(1..4).flat_map(move |(dir, step)| {
            let node = Node(row as isize, col as isize, dir, step);
            node.edges(array)
        })
    }).chain(Node(0, 0, Direction::Right, 0).edges(array))
}

fn main() -> Result<()> {
    let mut args = std::env::args();
    let fname = args.nth(1).ok_or_else(|| eyre!("filename was not provided"))?;
    let body = std::fs::read_to_string(fname.as_str())?;
    let blocks = parse_array(body.lines())?;
    let graph = Graph::<(), usize, Directed, usize>::from_edges(all_edges(&blocks));
    let (rows, cols) = blocks.dim();
    println!("({rows},{cols})");
    let min_finish = Node(rows as isize - 1, cols as isize - 1, Direction::Right, 0).id(rows, cols);
    let max_finish = Node(rows as isize - 1, cols as isize - 1, Direction::Up, 3).id(rows, cols);
    let sol = astar(&graph, 0.into(), |n| n >= min_finish.into() && n <= max_finish.into(), |e| *e.weight(), |_| 0);
    for node_id in sol.as_ref().unwrap().1.iter() {
        println!("{:?}", Node::from_id(node_id.index(), cols));
    }
    println!("{}", sol.as_ref().unwrap().0);
    Ok(())
}

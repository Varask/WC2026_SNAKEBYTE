use std::collections::{HashMap, HashSet, VecDeque};
use std::io::{self, BufRead};

// === Lecture stdin =============================================================

fn read_line() -> String {
    let stdin = io::stdin();
    let mut line = String::new();
    stdin.lock().read_line(&mut line).unwrap();
    line.trim().to_string()
}

fn read_int() -> i32 {
    read_line().parse().unwrap()
}

fn read_ints() -> Vec<i32> {
    read_line()
        .split_whitespace()
        .map(|x| x.parse().unwrap())
        .collect()
}

// === Types de base =============================================================

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
struct Pos {
    x: usize,
    y: usize,
}

impl Pos {
    fn new(x: usize, y: usize) -> Self {
        Self { x, y }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Dir {
    Up,
    Down,
    Left,
    Right,
}

impl Dir {
    fn to_str(self) -> &'static str {
        match self {
            Dir::Up    => "UP",
            Dir::Down  => "DOWN",
            Dir::Left  => "LEFT",
            Dir::Right => "RIGHT",
        }
    }

    fn apply(self, pos: Pos, width: usize, height: usize) -> Option<Pos> {
        let (nx, ny) = match self {
            Dir::Up    => (pos.x as isize,     pos.y as isize - 1),
            Dir::Down  => (pos.x as isize,     pos.y as isize + 1),
            Dir::Left  => (pos.x as isize - 1, pos.y as isize),
            Dir::Right => (pos.x as isize + 1, pos.y as isize),
        };
        if nx >= 0 && ny >= 0 && (nx as usize) < width && (ny as usize) < height {
            Some(Pos::new(nx as usize, ny as usize))
        } else {
            None
        }
    }

    fn all() -> [Dir; 4] {
        [Dir::Up, Dir::Down, Dir::Left, Dir::Right]
    }
}

// === Parse =====================================================================

fn parse_body(body: &str) -> Vec<Pos> {
    body.split(':')
        .map(|seg| {
            let mut it = seg.split(',');
            let x = it.next().unwrap().parse().unwrap();
            let y = it.next().unwrap().parse().unwrap();
            Pos::new(x, y)
        })
        .collect()
}

// === World ====================================================================

struct World {
    width:  usize,
    height: usize,
    walls:  HashSet<Pos>,
    // add the calques of the snakes (dictionnary id -> list of pos 1st is the head)
}

impl World {
    fn new(width: usize, height: usize, grid: &[String]) -> Self {
        let mut walls = HashSet::new();
        for (y, row) in grid.iter().enumerate() {
            for (x, c) in row.chars().enumerate() {
                if c == '#' {
                    walls.insert(Pos::new(x, y));
                }
            }
        }
        Self { width, height, walls }
    }

    fn is_wall(&self, pos: Pos) -> bool {
        self.walls.contains(&pos)
    }
}


// === Main =====================================================================

fn main() {
    let _my_id = read_int();
    let width   = read_int() as usize;
    let height  = read_int() as usize;

    let grid: Vec<String> = (0..height).map(|_| read_line()).collect();
    eprintln!("Grid {}x{}:", width, height);
    for row in &grid { eprintln!("{}", row); }

    let world = World::new(width, height, &grid);

    let snakebots_per_player = read_int();
    let my_snakebot_ids:  Vec<i32> = (0..snakebots_per_player).map(|_| read_int()).collect();
    let opp_snakebot_ids: Vec<i32> = (0..snakebots_per_player).map(|_| read_int()).collect();

    eprintln!("My snakebots:  {:?}", my_snakebot_ids);
    eprintln!("Opp snakebots: {:?}", opp_snakebot_ids);

    loop {
        // --- Sources d'energie ------------------------------------------------
        let power_source_count = read_int();
        let power_sources: Vec<Pos> = (0..power_source_count)
            .map(|_| { let v = read_ints(); Pos::new(v[0] as usize, v[1] as usize) })
            .collect();
        eprintln!("Power sources: {:?}", power_sources);

        // --- Snakebots --------------------------------------------------------
        let snakebot_count = read_int();
        let snakebots: Vec<(i32, Vec<Pos>)> = (0..snakebot_count)
            .map(|_| {
                let line = read_line();
                let mut parts = line.splitn(2, ' ');
                let id   = parts.next().unwrap().parse().unwrap();
                let body = parse_body(parts.next().unwrap());
                (id, body)
            })
            .collect();


        println!("WAIT");
  
    }
}
use std::io::{self, BufRead};

// Functions to read input from stdin
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
    read_line().split_whitespace()
        .map(|x| x.parse().unwrap())
        .collect()
}

// Define enums for actions that the snake can take
enum Actions {
    Up,
    Down,
    Left,
    Right,
    Wait,
}

// Define an enum for the type of block in the grid
enum BlockType {
    Empty,
    Wall,
}

fn char_to_block(c: char) -> BlockType {
    match c {
        '.' => BlockType::Empty,
        '#' => BlockType::Wall,
        _ => panic!("Unknown block type: {}", c),
    }
}

fn go(action: Actions) {
    let action_str = match action {
        Actions::Up    => "UP",
        Actions::Down  => "DOWN",
        Actions::Left  => "LEFT",
        Actions::Right => "RIGHT",
        Actions::Wait  => "WAIT",
    };
    println!("{}", action_str);
}

fn make_mark(x: i32, y: i32) -> String {
    format!("MARK {} {}", x, y)
}

// Une map 2D représentant la présence d'un élément sur chaque case
type Overlay = Vec<Vec<bool>>;

fn empty_overlay(width: usize, height: usize) -> Overlay {
    vec![vec![false; width]; height]
}

// Parse le body "x1,y1:x2,y2:..." en liste de coordonnées
fn parse_body(body: &str) -> Vec<(usize, usize)> {
    body.split(':')
        .map(|segment| {
            let mut coords = segment.split(',');
            let x = coords.next().unwrap().parse().unwrap();
            let y = coords.next().unwrap().parse().unwrap();
            (x, y)
        })
        .collect()
}

// Retourne pour chaque snake : (id, overlay du corps, coordonnées de la tête)
fn create_snake_ovs(
    snakebots: &[(i32, String)],
    width: usize,
    height: usize,
) -> Vec<(i32, Overlay, (usize, usize))> {
    snakebots.iter().map(|(id, body)| {
        let mut overlay = empty_overlay(width, height);
        let coords = parse_body(body);

        for &(x, y) in &coords {
            overlay[y][x] = true;
        }

        let head = coords[0];
        (*id, overlay, head)
    }).collect()
}

// Crée un overlay avec la position de toutes les sources de pouvoir
fn create_power_source_ov(
    power_sources: &[(i32, i32)],
    width: usize,
    height: usize,
) -> Overlay {
    let mut overlay = empty_overlay(width, height);

    for &(x, y) in power_sources {
        overlay[y as usize][x as usize] = true;
    }

    overlay
}


fn main() {
    let my_id = read_int();
    let width = read_int();
    let height = read_int();

    let grid: Vec<String> = (0..height).map(|_| read_line()).collect();
    eprintln!("Grid:");
    for line in &grid {
        eprintln!("{}", line);
    }

    let snakebots_per_player = read_int();
    let my_snakebots: Vec<i32>  = (0..snakebots_per_player).map(|_| read_int()).collect();
    let opp_snakebots: Vec<i32> = (0..snakebots_per_player).map(|_| read_int()).collect();

    eprintln!("My Snakebots: {:?}", my_snakebots);
    eprintln!("Opponent Snakebots: {:?}", opp_snakebots);

    loop {
        let power_source_count = read_int();
        let power_sources: Vec<(i32, i32)> = (0..power_source_count).map(|_| {
            let v = read_ints();
            (v[0], v[1])
        }).collect();
        eprintln!("Power Sources: {:?}", power_sources);

        let snakebot_count = read_int();
        let snakebots: Vec<(i32, String)> = (0..snakebot_count).map(|_| {
            let v = read_line();
            let mut parts = v.splitn(2, ' ');
            let id   = parts.next().unwrap().parse().unwrap();
            let body = parts.next().unwrap().to_string();
            (id, body)
        }).collect();

        for (id, body) in &snakebots {
            eprintln!("ID: {}, Body: {}", id, body);
        }

        // Création des overlays
        let snake_overlays  = create_snake_ovs(&snakebots, width as usize, height as usize);
        let power_overlay   = create_power_source_ov(&power_sources, width as usize, height as usize);

        // Debug : afficher la tête de chaque snake
        for (id, _overlay, head) in &snake_overlays {
            if my_snakebots.contains(id) {
                eprintln!("My snake {} head at {:?}", id, head);
            } else {
                eprintln!("Opp snake {} head at {:?}", id, head);
            }
        }

        go(Actions::Wait);
    }
}
use std::io::{self, BufRead};

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

enum Actions {
    Up,
    Down,
    Left,
    Right,
    Wait,

}

enum BlockType {
    Empty,
    Wall,
    PowerSource,
    SnakeBody(i32), // snake ID
}

fn go(action: Actions) {
    let action_str = match action {
        Actions::Up => "UP",
        Actions::Down => "DOWN",
        Actions::Left => "LEFT",
        Actions::Right => "RIGHT",
        Actions::Wait => "WAIT",
    };
    println!("{}", action_str);
}

fn make_mark(x, y: i32) -> String {
    format!("MARK {} {}", x, y)
}

// afin de ne pas corromple la matrice de la map; on va faire un calque contenant les infos des serpents (alliés et ennemis) et des sources de puissance
fn create_snake_ov() {

}

fn create_power_source_ov(){

}


fn main() {
    let my_id = read_int();
    let width = read_int();
    let height = read_int();

    let grid: Vec<String> = (0..height).map(|_| read_line()).collect();
    // print the content of the grid to stderr for debugging
    eprintln!("Grid:");
    for line in &grid {
        eprintln!("{}", line);
    }



    let snakebots_per_player = read_int();
    let my_snakebots: Vec<i32> = (0..snakebots_per_player).map(|_| read_int()).collect();
    let opp_snakebots: Vec<i32> = (0..snakebots_per_player).map(|_| read_int()).collect();


    
    eprintln!("My Snakebots: {:?}", my_snakebots);
    eprintln!("Opponent Snakebots: {:?}", opp_snakebots);


    loop {
        let power_source_count = read_int();
        let power_sources: Vec<(i32, i32)> = (0..power_source_count).map(|_| {
            let v = read_ints();
            (v[0], v[1])
        }).collect();

        let snakebot_count = read_int();
        let snakebots: Vec<(i32, String)> = (0..snakebot_count).map(|_| {
            let v = read_line();
            let mut parts = v.splitn(2, ' ');
            let id = parts.next().unwrap().parse().unwrap();
            let body = parts.next().unwrap().to_string();
            (id, body)
        }).collect();

        println!("WAIT");
    }
}
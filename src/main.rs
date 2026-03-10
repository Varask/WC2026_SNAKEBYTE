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

fn main() {
    let my_id = read_int();
    let width = read_int();
    let height = read_int();

    let grid: Vec<String> = (0..height).map(|_| read_line()).collect();

    let snakebots_per_player = read_int();
    let my_snakebots: Vec<i32> = (0..snakebots_per_player).map(|_| read_int()).collect();
    let opp_snakebots: Vec<i32> = (0..snakebots_per_player).map(|_| read_int()).collect();

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
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

    fn distance(self, other: Pos) -> usize {
        ((self.x as isize - other.x as isize).abs() + (self.y as isize - other.y as isize).abs()) as usize
    }

    fn neighbors(self, world: &World) -> Vec<(Dir, Pos)> { 
        Dir::all().iter()
            .filter_map(|&dir| dir.apply(self, world.width, world.height).map(|pos| (dir, pos)))
            .filter(|(_, pos)| !world.is_wall(*pos))
            .collect()
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
    wall_islands: Vec<Vec<Pos>>,      // îlots connexes de murs (calculé 1 fois)
    snakebots: HashMap<i32, Vec<Pos>>,
    power_sources: Vec<Pos>,
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
        let wall_islands = Self::compute_wall_islands(&walls, width, height);
        Self { width, height, walls, wall_islands, snakebots: HashMap::new(), power_sources: Vec::new() }
    }

    fn compute_wall_islands(walls: &HashSet<Pos>, width: usize, height: usize) -> Vec<Vec<Pos>> {
        let mut visited: HashSet<Pos> = HashSet::new();
        let mut islands = Vec::new();

        for &pos in walls {
            if visited.contains(&pos) { continue; }

            let mut island = Vec::new();
            let mut queue = VecDeque::new();
            queue.push_back(pos);
            visited.insert(pos);

            while let Some(cur) = queue.pop_front() {
                island.push(cur);
                for dir in Dir::all() {
                    if let Some(neighbor) = dir.apply(cur, width, height) {
                        if walls.contains(&neighbor) && !visited.contains(&neighbor) {
                            visited.insert(neighbor);
                            queue.push_back(neighbor);
                        }
                    }
                }
            }                  // ← ferme le while
            islands.push(island);
        }
        islands
    }

    fn is_wall(&self, pos: Pos) -> bool {
        self.walls.contains(&pos)
    }

    fn free_neighbors(&self, pos: Pos) -> Vec<Pos> {
        pos.neighbors(self)
            .into_iter()
            .map(|(_, p)| p)
            .filter(|p| !self.is_occupied(*p))
            .collect()
    }

    // Vrai si un snakebot occupe cette case
    fn is_occupied(&self, pos: Pos) -> bool {
        self.snakebots.values().any(|body| body.contains(&pos))
    }

    // Flood fill depuis pos → nombre de cases libres accessibles
    fn flood_fill(&self, start: Pos) -> usize {
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        queue.push_back(start);
        visited.insert(start);

        while let Some(cur) = queue.pop_front() {
            for n in self.free_neighbors(cur) {
                if !visited.contains(&n) {
                    visited.insert(n);
                    queue.push_back(n);
                }
            }
        }
        visited.len()
    }

    fn update_snakebots(&mut self, snakebots: HashMap<i32, Vec<Pos>>) {
        self.snakebots = snakebots;
    }

    fn update_power_sources(&mut self, power_sources: Vec<Pos>) {
        self.power_sources = power_sources;
    }
}

// === Debug Print Utilities ==========================================================

// Generer une vis 2D du monde dans stderr (pour debug)
//   - overlay de chaques element (murs, snakesbots, sources d'energie)
//   - assemblage avec des des oppérateurs logiques (ex: si mur ET snakebot → afficher mur)

fn walls_map(world: &World) -> Vec<Vec<String>> {
    (0..world.height).map(|y| {
        (0..world.width).map(|x| {
            if world.is_wall(Pos::new(x, y)) { "#".to_string() } else { ".".to_string() }
        }).collect()
    }).collect()
}

fn snakes_map(world: &World) -> Vec<Vec<String>> {
    let mut map = vec![vec![".".to_string(); world.width]; world.height];
    for (id, body) in &world.snakebots {
        for (i, pos) in body.iter().enumerate() {
            map[pos.y][pos.x] = if i == 0 { format!("H{}", id) } else { format!("B{}", id) };
        }
    }
    map
}

fn power_sources_map(world: &World) -> Vec<Vec<String>> {
    let mut map = vec![vec![".".to_string(); world.width]; world.height];
    for pos in &world.power_sources {
        map[pos.y][pos.x] = "P".to_string();
    }
    map
}

fn eprint_walls_map(world: &World) {
    let walls = walls_map(world);
    for row in walls {
        eprintln!("{}", row.join(""));
    }
}

fn eprint_full_world(world: &World) {
    let walls = walls_map(world);
    let snakes = snakes_map(world);
    let powers = power_sources_map(world);

    for y in 0..world.height {
        let mut line = String::new();
        for x in 0..world.width {
            line.push_str(&powers[y][x]);
            line.push_str(&snakes[y][x]);
            line.push_str(&walls[y][x]);
        }
        eprintln!("{}", line);
    }
}



// === Main =====================================================================

fn main() {
    // --- Initialisation ------------------------------------------------------
    let _my_id = read_int();
    let width   = read_int() as usize;
    let height  = read_int() as usize;
    let grid: Vec<String> = (0..height).map(|_| read_line()).collect();

    let mut world = World::new(width, height, &grid);

    eprint_full_world(&world);

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
        
        world.update_power_sources(power_sources);
        // --- Snakebots --------------------------------------------------------
        let snakebot_count = read_int();

        // Parse snakebots: map of snake ID to body positions (head first)
        let snakebots: HashMap<i32, Vec<Pos>> = (0..snakebot_count)
            .map(|_| {
                let line = read_line();  // lire toute la ligne d'un coup
                let mut parts = line.splitn(2, ' ');
                let id: i32 = parts.next().unwrap().parse().unwrap();
                let body = parts.next().unwrap();
                (id, parse_body(body))
            })
            .collect();
        
        world.update_snakebots(snakebots);

        eprint_full_world(&world);


        println!("WAIT");
  
    }
}

/*
========== PARTIE A NE PAS INCLURE DANS LE SCRIPT FINAL CODINGAME ==========
- Lecture des fichiers JSON de maps pour tester localement sans passer par le moteur de jeu.
- Tests unitaires pour les fonctions de base (Pos, Dir, World) afin de vérifier leur bon fonctionnement indépendamment du moteur de jeu.

*/
// === JSON Map Loader =========================================================
fn decode_rle_row(row: &str) -> String {
    let mut result = String::new();
    let mut num_buf = String::new();

    for c in row.chars() {
        if c.is_ascii_digit() {
            num_buf.push(c);
        } else {
            let count = if num_buf.is_empty() {
                1
            } else {
                num_buf.parse::<usize>().unwrap()
            };
            num_buf.clear();
            for _ in 0..count {
                result.push(c);
            }
        }
    }
    result
}

fn load_map_from_json(filepath: &str) -> World {
    let file_content = std::fs::read_to_string(filepath).expect("Failed to read map file");
    let json: serde_json::Value = serde_json::from_str(&file_content).expect("Failed to parse JSON");

    let dimensions = json["dimensions"].as_array().expect("Missing dimensions");
    let width  = dimensions[0].as_u64().unwrap() as usize;
    let height = dimensions[1].as_u64().unwrap() as usize;

    let grid_str = json["grid"].as_str().expect("Missing grid");
    let grid: Vec<String> = grid_str
        .split(';')
        .map(|row| decode_rle_row(row))
        .collect();

    assert_eq!(grid.len(), height, "Row count mismatch");
    assert!(grid.iter().all(|r| r.len() == width), "Column width mismatch");

    World::new(width, height, &grid)
}

// === Tests unitaires =========================================================

#[cfg(test)]
mod pos_tests {
    use super::*;

    fn make_empty_world(width: usize, height: usize) -> World {
        let grid: Vec<String> = (0..height).map(|_| ".".repeat(width)).collect();
        World::new(width, height, &grid)
    }

    #[test]
    fn test_distance_same() {
        let a = Pos::new(3, 3);
        assert_eq!(a.distance(a), 0);
    }

    #[test]
    fn test_distance_horizontal() {
        assert_eq!(Pos::new(0, 0).distance(Pos::new(5, 0)), 5);
    }

    #[test]
    fn test_distance_diagonal() {
        assert_eq!(Pos::new(0, 0).distance(Pos::new(3, 4)), 7);
    }

    #[test]
    fn test_neighbors_center() {
        let world = make_empty_world(5, 5);
        let neighbors = Pos::new(2, 2).neighbors(&world);
        assert_eq!(neighbors.len(), 4);
        let positions: Vec<Pos> = neighbors.iter().map(|(_, p)| *p).collect();
        assert!(positions.contains(&Pos::new(2, 1))); // Up
        assert!(positions.contains(&Pos::new(2, 3))); // Down
        assert!(positions.contains(&Pos::new(1, 2))); // Left
        assert!(positions.contains(&Pos::new(3, 2))); // Right
    }

    #[test]
    fn test_neighbors_corner() {
        // Coin haut-gauche : seulement 2 voisins valides
        let world = make_empty_world(5, 5);
        let neighbors = Pos::new(0, 0).neighbors(&world);
        assert_eq!(neighbors.len(), 2);
    }

    #[test]
    fn test_neighbors_edge() {
        // Bord gauche : seulement 3 voisins valides
        let world = make_empty_world(5, 5);
        let neighbors = Pos::new(0, 2).neighbors(&world);
        assert_eq!(neighbors.len(), 3);
    }

    #[test]
    fn test_neighbors_wall_excluded() {
        // Mur au-dessus : neighbors ne doit pas inclure la case mur
        let grid = vec![
            "#####".to_string(),
            ".....".to_string(),
            ".....".to_string(),
        ];
        let world = World::new(5, 3, &grid);
        let neighbors = Pos::new(2, 1).neighbors(&world);
        // Up → (2,0) est un mur, donc seulement 3 voisins
        assert_eq!(neighbors.len(), 3);
        let positions: Vec<Pos> = neighbors.iter().map(|(_, p)| *p).collect();
        assert!(!positions.contains(&Pos::new(2, 0)));
    }
}

#[cfg(test)]
mod dir_test {
    use super::*;

    #[test]
    fn test_apply_up() {
        assert_eq!(Dir::Up.apply(Pos::new(2, 2), 5, 5), Some(Pos::new(2, 1)));
    }

    #[test]
    fn test_apply_down() {
        assert_eq!(Dir::Down.apply(Pos::new(2, 2), 5, 5), Some(Pos::new(2, 3)));
    }

    #[test]
    fn test_apply_left() {
        assert_eq!(Dir::Left.apply(Pos::new(2, 2), 5, 5), Some(Pos::new(1, 2)));
    }

    #[test]
    fn test_apply_right() {
        assert_eq!(Dir::Right.apply(Pos::new(2, 2), 5, 5), Some(Pos::new(3, 2)));
    }

    #[test]
    fn test_apply_out_of_bounds_top() {
        assert_eq!(Dir::Up.apply(Pos::new(0, 0), 5, 5), None);
    }

    #[test]
    fn test_apply_out_of_bounds_left() {
        assert_eq!(Dir::Left.apply(Pos::new(0, 0), 5, 5), None);
    }

    #[test]
    fn test_apply_out_of_bounds_bottom() {
        assert_eq!(Dir::Down.apply(Pos::new(0, 4), 5, 5), None);
    }

    #[test]
    fn test_apply_out_of_bounds_right() {
        assert_eq!(Dir::Right.apply(Pos::new(4, 0), 5, 5), None);
    }

    #[test]
    fn test_all_has_4_dirs() {
        assert_eq!(Dir::all().len(), 4);
    }
}

#[cfg(test)]
mod world_tests {
    use super::*;

    fn make_world(grid: Vec<&str>) -> World {
        let h = grid.len();
        let w = grid[0].len();
        World::new(w, h, &grid.iter().map(|s| s.to_string()).collect::<Vec<_>>())
    }

    // ── Wall islands ──────────────────────────────────────────────────────────

    #[test]
    fn test_no_walls_no_islands() {
        let world = make_world(vec![
            ".....",
            ".....",
        ]);
        assert_eq!(world.wall_islands.len(), 0);
    }

    #[test]
    fn test_single_wall_one_island() {
        let world = make_world(vec![
            "..#..",
            ".....",
        ]);
        assert_eq!(world.wall_islands.len(), 1);
        assert_eq!(world.wall_islands[0].len(), 1);
    }

    #[test]
    fn test_two_separate_islands() {
        let world = make_world(vec![
            "#...#",
            ".....",
        ]);
        assert_eq!(world.wall_islands.len(), 2);
    }

    #[test]
    fn test_one_connected_island() {
        let world = make_world(vec![
            "###",
            ".#.",
        ]);
        assert_eq!(world.wall_islands.len(), 1);
        assert_eq!(world.wall_islands[0].len(), 4);
    }

    #[test]
    fn test_l_shaped_island() {
        // L-shape : 3 cases connectées
        let world = make_world(vec![
            "#..",
            "#..",
            "##.",
        ]);
        assert_eq!(world.wall_islands.len(), 1);
        assert_eq!(world.wall_islands[0].len(), 4);
    }

    // ── is_wall ───────────────────────────────────────────────────────────────

    #[test]
    fn test_is_wall_true() {
        let world = make_world(vec!["#.."]);
        assert!(world.is_wall(Pos::new(0, 0)));
    }

    #[test]
    fn test_is_wall_false() {
        let world = make_world(vec!["#.."]);
        assert!(!world.is_wall(Pos::new(1, 0)));
    }

    // ── flood_fill ────────────────────────────────────────────────────────────

    #[test]
    fn test_flood_fill_open() {
        // Grille 3x3 vide → flood fill depuis centre = 9 cases
        let world = make_world(vec![
            "...",
            "...",
            "...",
        ]);
        assert_eq!(world.flood_fill(Pos::new(1, 1)), 9);
    }

    #[test]
    fn test_flood_fill_walled_in() {
        // Case isolée par des murs
        let world = make_world(vec![
            "###",
            "#.#",
            "###",
        ]);
        assert_eq!(world.flood_fill(Pos::new(1, 1)), 1);
    }

    #[test]
    fn test_flood_fill_corridor() {
        // Couloir horizontal : 3 cases libres
        let world = make_world(vec![
            "#####",
            "#...#",
            "#####",
        ]);
        assert_eq!(world.flood_fill(Pos::new(2, 1)), 3);
    }

    #[test]
    fn test_flood_fill_split_zones() {
        // Mur vertical sépare deux zones
        let world = make_world(vec![
            "..#..",
            "..#..",
            "..#..",
        ]);
        // Zone gauche = 6 cases, zone droite = 6 cases
        assert_eq!(world.flood_fill(Pos::new(0, 0)), 6);
        assert_eq!(world.flood_fill(Pos::new(4, 0)), 6);
    }

    // ── is_occupied ───────────────────────────────────────────────────────────

    #[test]
    fn test_is_occupied_empty() {
        let world = make_world(vec!["..."]);
        assert!(!world.is_occupied(Pos::new(0, 0)));
    }

    #[test]
    fn test_is_occupied_with_snake() {
        let mut world = make_world(vec!["..."]);
        let mut snakebots = std::collections::HashMap::new();
        snakebots.insert(0, vec![Pos::new(1, 0), Pos::new(0, 0)]);
        world.update_snakebots(snakebots);
        assert!(world.is_occupied(Pos::new(1, 0)));
        assert!(world.is_occupied(Pos::new(0, 0)));
        assert!(!world.is_occupied(Pos::new(2, 0)));
    }
}
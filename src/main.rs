use std::collections::{HashMap, HashSet, VecDeque};
use std::io::{self, BufRead};

// ─── Lecture stdin ──────────────────────────────────────────────────────────

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

// ─── Types de base ───────────────────────────────────────────────────────────

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

    /// Applique la direction à une position, retourne None si hors-grille
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

// ─── Parse ───────────────────────────────────────────────────────────────────

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

// ─── État du monde ───────────────────────────────────────────────────────────

struct World {
    width:  usize,
    height: usize,
    walls:  HashSet<Pos>,
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

// ─── BFS ─────────────────────────────────────────────────────────────────────

/// Retourne la première direction à prendre depuis `start` pour atteindre `goal`,
/// en évitant les cases bloquées (`blocked`).
/// Retourne None si aucun chemin n'existe.
fn bfs(
    start:   Pos,
    goal:    Pos,
    world:   &World,
    blocked: &HashSet<Pos>,
) -> Option<Dir> {
    if start == goal {
        return None; // déjà sur l'objectif
    }

    let mut visited: HashSet<Pos> = HashSet::new();
    // (position, première direction prise depuis start)
    let mut queue: VecDeque<(Pos, Dir)> = VecDeque::new();

    visited.insert(start);

    for &dir in &Dir::all() {
        if let Some(next) = dir.apply(start, world.width, world.height) {
            if !world.is_wall(next) && !blocked.contains(&next) && !visited.contains(&next) {
                visited.insert(next);
                queue.push_back((next, dir));
            }
        }
    }

    while let Some((pos, first_dir)) = queue.pop_front() {
        if pos == goal {
            return Some(first_dir);
        }
        for &dir in &Dir::all() {
            if let Some(next) = dir.apply(pos, world.width, world.height) {
                if !world.is_wall(next) && !blocked.contains(&next) && !visited.contains(&next) {
                    visited.insert(next);
                    queue.push_back((next, first_dir));
                }
            }
        }
    }

    None // pas de chemin
}

/// Distance BFS depuis `start` vers `goal` (pour comparer les distances entre snakes).
fn bfs_distance(
    start:   Pos,
    goal:    Pos,
    world:   &World,
    blocked: &HashSet<Pos>,
) -> Option<u32> {
    if start == goal {
        return Some(0);
    }

    let mut visited: HashSet<Pos> = HashSet::new();
    let mut queue: VecDeque<(Pos, u32)> = VecDeque::new();

    visited.insert(start);
    queue.push_back((start, 0));

    while let Some((pos, dist)) = queue.pop_front() {
        for &dir in &Dir::all() {
            if let Some(next) = dir.apply(pos, world.width, world.height) {
                if !world.is_wall(next) && !blocked.contains(&next) && !visited.contains(&next) {
                    if next == goal {
                        return Some(dist + 1);
                    }
                    visited.insert(next);
                    queue.push_back((next, dist + 1));
                }
            }
        }
    }

    None
}

// ─── Attribution des objectifs ────────────────────────────────────────────────

/// Assigne chaque snake "my" à une source d'énergie différente.
/// Chaque source ne peut être attribuée qu'à un seul snake.
/// Retourne une map snake_id → position cible.
fn assign_objectives(
    my_snakes: &[(i32, Pos)],      // (id, tête)
    power_sources: &[Pos],
    world: &World,
    blocked: &HashSet<Pos>,
) -> HashMap<i32, Pos> {
    let mut assignments: HashMap<i32, Pos> = HashMap::new();
    let mut taken: HashSet<Pos> = HashSet::new();

    // Trier les snakes par id pour un ordre déterministe
    let mut snakes = my_snakes.to_vec();
    snakes.sort_by_key(|(id, _)| *id);

    for (id, head) in &snakes {
        // Trouver la source la plus proche non encore prise
        let best = power_sources
            .iter()
            .filter(|&&ps| !taken.contains(&ps))
            .min_by_key(|&&ps| {
                bfs_distance(*head, ps, world, blocked).unwrap_or(u32::MAX)
            });

        if let Some(&target) = best {
            assignments.insert(*id, target);
            taken.insert(target);
        }
    }

    assignments
}

// ─── Direction de repli (éviter les murs/corps) ───────────────────────────────

/// Si le pathfinding échoue, choisit une direction sûre au hasard (première valide).
fn fallback_dir(head: Pos, world: &World, blocked: &HashSet<Pos>) -> Option<Dir> {
    Dir::all().iter().find(|&&dir| {
        if let Some(next) = dir.apply(head, world.width, world.height) {
            !world.is_wall(next) && !blocked.contains(&next)
        } else {
            false
        }
    }).copied()
}

// ─── Sortie ───────────────────────────────────────────────────────────────────

fn output_action(snake_id: i32, dir: Option<Dir>) {
    match dir {
        Some(d) => println!("{} {}", snake_id, d.to_str()),
        None    => println!("WAIT"),
    }
}

// ─── Main ─────────────────────────────────────────────────────────────────────

fn main() {
    let my_id = read_int();
    let width  = read_int() as usize;
    let height = read_int() as usize;

    let grid: Vec<String> = (0..height).map(|_| read_line()).collect();
    eprintln!("Grid {}x{}:", width, height);
    for row in &grid {
        eprintln!("{}", row);
    }

    let world = World::new(width, height, &grid);

    let snakebots_per_player = read_int();
    let my_snakebot_ids:  Vec<i32> = (0..snakebots_per_player).map(|_| read_int()).collect();
    let opp_snakebot_ids: Vec<i32> = (0..snakebots_per_player).map(|_| read_int()).collect();

    eprintln!("My snakebots:  {:?}", my_snakebot_ids);
    eprintln!("Opp snakebots: {:?}", opp_snakebot_ids);

    // Objectifs persistants entre les tours : snake_id → cible actuelle
    let mut current_objectives: HashMap<i32, Pos> = HashMap::new();

    loop {
        // ── Lecture des sources d'énergie ─────────────────────────────────
        let power_source_count = read_int();
        let power_sources: Vec<Pos> = (0..power_source_count)
            .map(|_| {
                let v = read_ints();
                Pos::new(v[0] as usize, v[1] as usize)
            })
            .collect();
        eprintln!("Power sources: {:?}", power_sources);

        // ── Lecture des snakebots ─────────────────────────────────────────
        let snakebot_count = read_int();
        let snakebots_raw: Vec<(i32, String)> = (0..snakebot_count)
            .map(|_| {
                let line = read_line();
                let mut parts = line.splitn(2, ' ');
                let id   = parts.next().unwrap().parse().unwrap();
                let body = parts.next().unwrap().to_string();
                (id, body)
            })
            .collect();

        // Parse corps de chaque snake
        let snakebots: Vec<(i32, Vec<Pos>)> = snakebots_raw
            .iter()
            .map(|(id, body)| (*id, parse_body(body)))
            .collect();

        // ── Construire l'ensemble des cases bloquées (tous les corps) ─────
        let mut blocked: HashSet<Pos> = HashSet::new();
        for (_, body) in &snakebots {
            for &pos in body {
                blocked.insert(pos);
            }
        }

        // ── Récupérer les têtes de mes snakes ─────────────────────────────
        let my_snakes: Vec<(i32, Pos)> = snakebots
            .iter()
            .filter(|(id, _)| my_snakebot_ids.contains(id))
            .map(|(id, body)| (*id, body[0]))
            .collect();

        // ── Invalider les objectifs déjà atteints ou disparus ─────────────
        let power_set: HashSet<Pos> = power_sources.iter().copied().collect();
        current_objectives.retain(|_, target| power_set.contains(target));

        // Invalider les objectifs partagés (deux snakes sur la même cible après une disparition)
        {
            let mut seen_targets: HashSet<Pos> = HashSet::new();
            let mut to_remove: Vec<i32> = Vec::new();
            for (id, target) in &current_objectives {
                if !seen_targets.insert(*target) {
                    to_remove.push(*id);
                }
            }
            for id in to_remove {
                current_objectives.remove(&id);
            }
        }

        // ── Identifier les snakes sans objectif ───────────────────────────
        let unassigned: Vec<(i32, Pos)> = my_snakes
            .iter()
            .filter(|(id, _)| !current_objectives.contains_key(id))
            .cloned()
            .collect();

        // Sources encore libres (non attribuées)
        let assigned_targets: HashSet<Pos> = current_objectives.values().copied().collect();
        let free_sources: Vec<Pos> = power_sources
            .iter()
            .filter(|&&ps| !assigned_targets.contains(&ps))
            .copied()
            .collect();

        // Attribuer de nouveaux objectifs aux snakes non assignés
        let new_assignments = assign_objectives(&unassigned, &free_sources, &world, &blocked);
        for (id, target) in new_assignments {
            eprintln!("Snake {} → new objective {:?}", id, target);
            current_objectives.insert(id, target);
        }

        // ── Calculer et émettre les actions ───────────────────────────────
        let mut actions: Vec<String> = Vec::new();

        for (id, head) in &my_snakes {
            let dir = if let Some(&target) = current_objectives.get(id) {
                eprintln!("Snake {} at {:?} → target {:?}", id, head, target);
                // Cases bloquées sauf la queue (la queue bougera avant)
                bfs(*head, target, &world, &blocked)
                    .or_else(|| fallback_dir(*head, &world, &blocked))
            } else {
                // Pas d'objectif disponible : direction sûre
                eprintln!("Snake {} at {:?}: no objective, fallback", id, head);
                fallback_dir(*head, &world, &blocked)
            };

            match dir {
                Some(d) => actions.push(format!("{} {}", id, d.to_str())),
                None    => {} // ne rien faire pour ce snake
            }
        }

        if actions.is_empty() {
            println!("WAIT");
        } else {
            println!("{}", actions.join(";"));
        }
    }
}
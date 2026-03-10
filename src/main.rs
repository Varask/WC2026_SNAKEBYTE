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

// === BFS ======================================================================

/// Retourne la premiere direction vers `goal` depuis `start`,
/// en evitant `blocked`. None si deja sur place ou pas de chemin.
fn bfs(
    start:   Pos,
    goal:    Pos,
    world:   &World,
    blocked: &HashSet<Pos>,
) -> Option<Dir> {
    if start == goal {
        return None;
    }

    let mut visited: HashSet<Pos> = HashSet::new();
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

    None
}

/// Distance BFS de `start` a `goal`. None si inaccessible.
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

// === Zones de danger ==========================================================

/// Cases dans lesquelles une tete ennemie peut se deplacer au prochain tour.
/// On inclut la tete elle-meme (collision frontale) et tous ses voisins libres.
fn enemy_danger_zone(enemy_heads: &[Pos], world: &World) -> HashSet<Pos> {
    let mut danger = HashSet::new();
    for &head in enemy_heads {
        danger.insert(head);
        for &dir in &Dir::all() {
            if let Some(next) = dir.apply(head, world.width, world.height) {
                if !world.is_wall(next) {
                    danger.insert(next);
                }
            }
        }
    }
    danger
}

// === Direction de repli =======================================================

/// Choisit la meilleure direction sure disponible.
/// Prefere les cases hors danger ennemi ; accepte le danger en dernier recours.
fn fallback_dir(
    head:    Pos,
    world:   &World,
    blocked: &HashSet<Pos>,
    danger:  &HashSet<Pos>,
) -> Option<Dir> {
    let safe: Vec<Dir> = Dir::all().iter().filter(|&&dir| {
        dir.apply(head, world.width, world.height)
            .map(|next| !world.is_wall(next) && !blocked.contains(&next))
            .unwrap_or(false)
    }).copied().collect();

    safe.iter()
        .find(|&&dir| {
            dir.apply(head, world.width, world.height)
                .map(|next| !danger.contains(&next))
                .unwrap_or(false)
        })
        .or_else(|| safe.first())
        .copied()
}

// === Choix de direction avec evitement des collisions =========================

/// Choisit la direction optimale pour un snake en tenant compte :
///   - du chemin BFS vers la cible
///   - de la zone de danger ennemie (cases evitees en priorite)
///   - des reservations des snakes allies deja traites (dans `blocked`)
fn choose_dir(
    id:       i32,
    head:     Pos,
    own_body: &[Pos],
    target:   Option<Pos>,
    world:    &World,
    blocked:  &HashSet<Pos>,
    danger:   &HashSet<Pos>,
) -> Option<Dir> {
    // Blocked local : retire la tete et la queue propres
    let mut local_blocked = blocked.clone();
    local_blocked.remove(&head);
    if let Some(&tail) = own_body.last() {
        local_blocked.remove(&tail);
    }

    // 1) BFS vers la cible en evitant aussi la zone de danger
    let dir_safe = target.and_then(|goal| {
        let mut bd = local_blocked.clone();
        for &d in danger.iter() { bd.insert(d); }
        bfs(head, goal, world, &bd)
    });

    // 2) BFS vers la cible sans contrainte de danger (chemin de secours)
    let dir_any = target.and_then(|goal| bfs(head, goal, world, &local_blocked));

    // Choisir la meilleure option disponible
    let chosen = match dir_safe.or(dir_any) {
        Some(d) => {
            // Si la direction BFS atterrit dans le danger ET qu'un fallback
            // sur sans danger existe, le privilegier
            let next = d.apply(head, world.width, world.height);
            let in_danger = next.map(|p| danger.contains(&p)).unwrap_or(false);
            if in_danger {
                fallback_dir(head, world, &local_blocked, danger)
                    .map(|safe_d| {
                        eprintln!("Snake {} : BFS dangereux ({}), fallback -> {}", id, d.to_str(), safe_d.to_str());
                        safe_d
                    })
                    .unwrap_or_else(|| {
                        eprintln!("Snake {} : force dans le danger ({})", id, d.to_str());
                        d
                    })
            } else {
                d
            }
        }
        None => {
            eprintln!("Snake {} : pas de chemin BFS, fallback", id);
            fallback_dir(head, world, &local_blocked, danger)?
        }
    };

    Some(chosen)
}

// === Attribution des objectifs ================================================

fn assign_objectives(
    my_snakes:     &[(i32, Pos)],
    power_sources: &[Pos],
    world:         &World,
    blocked:       &HashSet<Pos>,
) -> HashMap<i32, Pos> {
    let mut assignments: HashMap<i32, Pos> = HashMap::new();
    let mut taken: HashSet<Pos> = HashSet::new();

    let mut snakes = my_snakes.to_vec();
    snakes.sort_by_key(|(id, _)| *id);

    for (id, head) in &snakes {
        let best = power_sources.iter()
            .filter(|&&ps| !taken.contains(&ps))
            .min_by_key(|&&ps| bfs_distance(*head, ps, world, blocked).unwrap_or(u32::MAX));

        if let Some(&target) = best {
            assignments.insert(*id, target);
            taken.insert(target);
        }
    }

    assignments
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

    let mut current_objectives: HashMap<i32, Pos> = HashMap::new();

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

        // --- Blocked global ---------------------------------------------------
        let mut blocked: HashSet<Pos> = HashSet::new();
        for (_, body) in &snakebots {
            for &pos in body { blocked.insert(pos); }
        }

        // --- Tetes ------------------------------------------------------------
        let my_snakes: Vec<(i32, Pos)> = snakebots.iter()
            .filter(|(id, _)| my_snakebot_ids.contains(id))
            .map(|(id, body)| (*id, body[0]))
            .collect();

        let enemy_heads: Vec<Pos> = snakebots.iter()
            .filter(|(id, _)| opp_snakebot_ids.contains(id))
            .map(|(_, body)| body[0])
            .collect();

        let danger = enemy_danger_zone(&enemy_heads, &world);

        // --- Invalidation des objectifs ---------------------------------------
        let power_set: HashSet<Pos> = power_sources.iter().copied().collect();
        current_objectives.retain(|_, target| power_set.contains(target));

        // Supprimer les doublons (garder le snake avec le plus petit ID)
        {
            let mut seen: HashSet<Pos> = HashSet::new();
            let mut to_remove: Vec<i32> = Vec::new();
            let mut pairs: Vec<(i32, Pos)> = current_objectives
                .iter().map(|(&id, &p)| (id, p)).collect();
            pairs.sort_by_key(|(id, _)| *id);
            for (id, target) in pairs {
                if !seen.insert(target) { to_remove.push(id); }
            }
            for id in to_remove { current_objectives.remove(&id); }
        }

        // --- Nouveaux objectifs -----------------------------------------------
        let unassigned: Vec<(i32, Pos)> = my_snakes.iter()
            .filter(|(id, _)| !current_objectives.contains_key(id))
            .cloned().collect();

        let assigned_targets: HashSet<Pos> = current_objectives.values().copied().collect();
        let free_sources: Vec<Pos> = power_sources.iter()
            .filter(|&&ps| !assigned_targets.contains(&ps))
            .copied().collect();

        let new_assignments = assign_objectives(&unassigned, &free_sources, &world, &blocked);
        for (id, target) in new_assignments {
            eprintln!("Snake {} -> objectif {:?}", id, target);
            current_objectives.insert(id, target);
        }

        // --- Actions avec reservation progressive ----------------------------
        // Chaque snake, une fois son mouvement decide, reserve sa future tete
        // dans working_blocked pour que les snakes traites apres l'evitent.
        let mut actions: Vec<String> = Vec::new();
        let mut working_blocked = blocked.clone();

        let mut ordered_snakes = my_snakes.clone();
        ordered_snakes.sort_by_key(|(id, _)| *id);

        for (id, head) in &ordered_snakes {
            let own_body: Vec<Pos> = snakebots.iter()
                .find(|(sid, _)| sid == id)
                .map(|(_, body)| body.clone())
                .unwrap_or_default();

            let target = current_objectives.get(id).copied();

            let dir = choose_dir(
                *id, *head, &own_body, target,
                &world, &working_blocked, &danger,
            );

            if let Some(d) = dir {
                if let Some(next_head) = d.apply(*head, world.width, world.height) {
                    // Reserver la future tete pour les snakes traites apres
                    working_blocked.insert(next_head);
                    eprintln!("Snake {} : {} (reserve {:?})", id, d.to_str(), next_head);
                }
                actions.push(format!("{} {}", id, d.to_str()));
            }
        }

        if actions.is_empty() {
            println!("WAIT");
        } else {
            println!("{}", actions.join(";"));
        }
    }
}
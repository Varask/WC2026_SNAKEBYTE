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
    // `blocked` a deja les queues retirees par l'appelant (working_blocked).
    // On retire quand meme la queue propre par securite (idempotent).
    // La tete NE doit PAS etre retiree : le BFS part de head via visited,
    // la retirer autoriserait a repasser sur la case actuelle (demi-tour).
    let mut local_blocked = blocked.clone();
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

// === Scoring des objectifs ====================================================

/// Distance de Manhattan entre deux positions (pas de BFS, O(1)).
fn manhattan(a: Pos, b: Pos) -> u32 {
    ((a.x as i32 - b.x as i32).abs() + (a.y as i32 - b.y as i32).abs()) as u32
}

/// BFS multi-sources : retourne une map Pos -> distance minimale depuis
/// n'importe laquelle des `origins`. Un seul BFS pour toutes les origines.
/// Complexite : O(W x H) quel que soit le nombre d'origines.
fn bfs_multi(
    origins: &[Pos],
    world:   &World,
    blocked: &HashSet<Pos>,
) -> HashMap<Pos, u32> {
    let mut dist: HashMap<Pos, u32> = HashMap::new();
    let mut queue: VecDeque<(Pos, u32)> = VecDeque::new();
    for &o in origins {
        if !blocked.contains(&o) && !world.is_wall(o) {
            dist.insert(o, 0);
            queue.push_back((o, 0));
        }
    }
    while let Some((pos, d)) = queue.pop_front() {
        for &dir in &Dir::all() {
            if let Some(next) = dir.apply(pos, world.width, world.height) {
                if !world.is_wall(next) && !blocked.contains(&next) && !dist.contains_key(&next) {
                    dist.insert(next, d + 1);
                    queue.push_back((next, d + 1));
                }
            }
        }
    }
    dist
}

/// Calcule le score de chaque source candidate pour chaque snake.
///
/// Toute la phase de scoring tient en un nombre fixe de BFS :
///   - 1 BFS par snake (distances snake -> toutes cases)
///   - 1 BFS multi-sources depuis toutes les tetes ennemies
/// Puis le cluster bonus utilise la distance de Manhattan (O(1) par paire).
///
/// Composantes du score (plus haut = plus attractif) :
///   cluster_bonus  : sources voisines dans un rayon Manhattan (densite locale)
///   dist_snake     : distance BFS du snake jusqu'a la source (penalite)
///   enemy_closer   : penalite si un ennemi arrive avant nous
fn compute_scores(
    my_snakes:     &[(i32, Pos)],
    power_sources: &[Pos],
    enemy_heads:   &[Pos],
    world:         &World,
    blocked:       &HashSet<Pos>,
) -> HashMap<(i32, Pos), i64> {
    const CLUSTER_RADIUS: u32 = 4;   // rayon Manhattan pour compter les voisines
    const CLUSTER_WEIGHT: i64 = 20;  // bonus par source voisine dans le rayon
    const DIST_WEIGHT:    i64 = 50;  // penalite par case de distance BFS (prio proximite)
    const ENEMY_PENALTY:  i64 = 80;  // penalite si ennemi plus proche

    // Un BFS par snake depuis sa tete
    let snake_dists: Vec<(i32, HashMap<Pos, u32>)> = my_snakes.iter()
        .map(|&(id, head)| (id, bfs_multi(&[head], world, blocked)))
        .collect();

    // Un seul BFS multi-sources pour tous les ennemis
    let enemy_dist_map = bfs_multi(enemy_heads, world, blocked);

    // Densite de chaque source : nombre de sources voisines a rayon Manhattan
    let cluster: HashMap<Pos, i64> = power_sources.iter()
        .map(|&src| {
            let count = power_sources.iter()
                .filter(|&&other| other != src && manhattan(src, other) <= CLUSTER_RADIUS)
                .count() as i64;
            (src, count)
        })
        .collect();

    // Assembler les scores
    let mut scores = HashMap::new();
    for &(id, head) in my_snakes {
        let dmap = snake_dists.iter().find(|(sid, _)| *sid == id).map(|(_, m)| m);
        for &src in power_sources {
            if let Some(dist_snake) = dmap.and_then(|m| m.get(&src)).copied() {
                let dist_snake = dist_snake as i64;
                let cluster_bonus = cluster.get(&src).copied().unwrap_or(0);
                let enemy_d = enemy_dist_map.get(&src).copied().unwrap_or(u32::MAX) as i64;
                let enemy_closer = !enemy_heads.is_empty() && enemy_d < dist_snake;
                let score = cluster_bonus * CLUSTER_WEIGHT
                    - dist_snake * DIST_WEIGHT
                    - if enemy_closer { ENEMY_PENALTY } else { 0 };
                scores.insert((id, src), score);
            }
            // Si non present dans dmap : inaccessible, pas insere => filtre naturel
        }
    }
    scores
}

// === Attribution des objectifs ================================================

fn assign_objectives(
    my_snakes:     &[(i32, Pos)],
    power_sources: &[Pos],
    enemy_heads:   &[Pos],
    world:         &World,
    blocked:       &HashSet<Pos>,
) -> HashMap<i32, Pos> {
    let scores = compute_scores(my_snakes, power_sources, enemy_heads, world, blocked);

    let mut assignments: HashMap<i32, Pos> = HashMap::new();
    let mut taken: HashSet<Pos> = HashSet::new();

    let mut snakes = my_snakes.to_vec();
    snakes.sort_by_key(|(id, _)| *id);

    for (id, _head) in &snakes {
        let best = power_sources.iter()
            .filter(|&&ps| !taken.contains(&ps))
            .filter_map(|&ps| scores.get(&(*id, ps)).map(|&s| (ps, s)))
            .max_by_key(|&(_, s)| s);

        if let Some((target, score)) = best {
            eprintln!("  Snake {} -> {:?} (score {})", id, target, score);
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

        // --- Invalidation des objectifs inaccessibles ------------------------
        // Un objectif est inaccessible si aucun chemin BFS n'existe depuis
        // la tete du snake jusqu'a la cible (sans contrainte de danger).
        {
            let mut to_invalidate: Vec<i32> = Vec::new();
            for (&id, &target) in &current_objectives {
                let head = match my_snakes.iter().find(|(sid, _)| *sid == id) {
                    Some((_, h)) => *h,
                    None => continue,
                };
                let own_body: Vec<Pos> = snakebots.iter()
                    .find(|(sid, _)| *sid == id)
                    .map(|(_, b)| b.clone())
                    .unwrap_or_default();
                let mut local_blocked = blocked.clone();
                if let Some(&tail) = own_body.last() {
                    local_blocked.remove(&tail);
                }
                if bfs_distance(head, target, &world, &local_blocked).is_none() {
                    eprintln!("Snake {} : objectif {:?} INACCESSIBLE, invalidation", id, target);
                    to_invalidate.push(id);
                }
            }
            for id in to_invalidate {
                current_objectives.remove(&id);
            }
        }

        // --- Nouveaux objectifs -----------------------------------------------
        let unassigned: Vec<(i32, Pos)> = my_snakes.iter()
            .filter(|(id, _)| !current_objectives.contains_key(id))
            .cloned().collect();

        let assigned_targets: HashSet<Pos> = current_objectives.values().copied().collect();
        let free_sources: Vec<Pos> = power_sources.iter()
            .filter(|&&ps| !assigned_targets.contains(&ps))
            .copied().collect();

        let new_assignments = assign_objectives(&unassigned, &free_sources, &enemy_heads, &world, &blocked);
        for (id, target) in new_assignments {
            eprintln!("Snake {} -> objectif {:?}", id, target);
            current_objectives.insert(id, target);
        }

        // --- Actions avec reservation progressive ----------------------------
        // working_blocked = tous les corps, queues retirees (elles se liberent).
        // On traite les snakes par ordre croissant d'ID. Apres chaque decision,
        // la future tete est inseree dans working_blocked pour que les snakes
        // suivants l'evitent => evite les collisions tete-a-tete entre allies.
        let mut actions: Vec<String> = Vec::new();

        // Base : tous les corps, queues retirees
        let mut working_blocked: HashSet<Pos> = HashSet::new();
        for (_sid, body) in &snakebots {
            for &pos in body { working_blocked.insert(pos); }
            if let Some(&tail) = body.last() { working_blocked.remove(&tail); }
        }

        // Passe 1 : estimer la prochaine position de chaque snake allie
        // (on suppose qu'il continue dans la direction de son objectif ou
        //  dans la premiere direction libre). Ces estimations sont inserees
        //  dans working_blocked AVANT la passe 2 pour que chaque snake
        //  planifie en evitant les futures tetes de tous ses allies.
        let mut ordered_snakes = my_snakes.clone();
        ordered_snakes.sort_by_key(|(id, _)| *id);

        let mut estimated_next: HashMap<i32, Pos> = HashMap::new();
        {
            let mut est_blocked = working_blocked.clone();
            for (id, head) in &ordered_snakes {
                let own_body: Vec<Pos> = snakebots.iter()
                    .find(|(sid, _)| sid == id)
                    .map(|(_, body)| body.clone())
                    .unwrap_or_default();
                let target = current_objectives.get(id).copied();
                // Estimation rapide : BFS sans contrainte danger
                let est_dir = target
                    .and_then(|goal| bfs(*head, goal, &world, &est_blocked))
                    .or_else(|| fallback_dir(*head, &world, &est_blocked, &danger));
                if let Some(d) = est_dir {
                    if let Some(next) = d.apply(*head, world.width, world.height) {
                        estimated_next.insert(*id, next);
                        est_blocked.insert(next); // bloquer pour les suivants deja en passe 1
                    }
                }
            }
        }
        // Ajouter toutes les estimations dans working_blocked
        for &next in estimated_next.values() {
            working_blocked.insert(next);
        }

        // Passe 2 : calcul definitif avec working_blocked complet
        for (id, head) in &ordered_snakes {
            // Retirer l'estimation de CE snake pour qu'il puisse planifier
            // sans se bloquer lui-meme
            if let Some(&own_est) = estimated_next.get(id) {
                working_blocked.remove(&own_est);
            }

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
                    // Remplacer l'estimation par la decision reelle
                    working_blocked.insert(next_head);
                    eprintln!("Snake {} : {} (next {:?})", id, d.to_str(), next_head);
                }
                actions.push(format!("{} {}", id, d.to_str()));
            } else {
                // Remettre l'estimation si ce snake ne bouge pas
                if let Some(&own_est) = estimated_next.get(id) {
                    working_blocked.insert(own_est);
                }
            }
        }

        if actions.is_empty() {
            println!("WAIT");
        } else {
            println!("{}", actions.join(";"));
        }
    }
}
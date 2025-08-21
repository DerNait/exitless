use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashSet};
use raylib::prelude::Vector2;
use crate::maze::Maze;
use crate::caster::is_passable;
use crate::utils_grid::{world_to_cell, cell_center};

#[derive(Debug)]
pub struct Enemy {
    pub pos: Vector2,          // posición en píxeles
    pub speed: f32,            // px/seg
    pub path: Vec<(i32,i32)>,  // camino en celdas (desde el siguiente paso)
    replan_accum: f32,
}

impl Enemy {
    pub fn from_cell(ci: i32, cj: i32, bs: usize) -> Self {
        Self {
            pos: cell_center(ci, cj, bs),
            speed: 45.0,
            path: Vec::new(),
            replan_accum: 0.0,
        }
    }
}

// --- A* ---
#[derive(Clone, Eq, PartialEq)]
struct Node { f: i32, g: i32, x: i32, y: i32 }
impl Ord for Node {
    fn cmp(&self, other: &Self) -> Ordering {
        other.f.cmp(&self.f).then_with(|| other.g.cmp(&self.g))
    }
}
impl PartialOrd for Node { fn partial_cmp(&self, o: &Self) -> Option<Ordering> { Some(self.cmp(o)) } }

fn manhattan(ax: i32, ay: i32, bx: i32, by: i32) -> i32 {
    (ax - bx).abs() + (ay - by).abs()
}

fn neighbors(x: i32, y: i32, maze: &Maze) -> impl Iterator<Item=(i32,i32)> + '_ {
    const OFFS: [(i32,i32);4] = [(1,0),(-1,0),(0,1),(0,-1)];
    OFFS.into_iter().filter_map(move |(dx,dy)| {
        let nx = x+dx; let ny = y+dy;
        if ny >= 0 && (ny as usize) < maze.len() &&
           nx >= 0 && (nx as usize) < maze[ny as usize].len() &&
           is_passable(maze[ny as usize][nx as usize]) {
            Some((nx,ny))
        } else { None }
    })
}

pub fn astar(maze: &Maze, start: (i32,i32), goal: (i32,i32)) -> Option<Vec<(i32,i32)>> {
    if start == goal { return Some(vec![]); }
    let (gx,gy) = goal;
    let mut open = BinaryHeap::new();
    let mut came: HashSet<(i32,i32)> = HashSet::new();
    let mut parent = std::collections::HashMap::new();
    let mut gscore = std::collections::HashMap::new();

    let h0 = manhattan(start.0, start.1, gx, gy);
    open.push(Node { f: h0, g:0, x: start.0, y: start.1 });
    gscore.insert(start, 0);

    while let Some(Node { g, x, y, .. }) = open.pop() {
        if (x,y) == goal {
            let mut path = Vec::new();
            let mut cur = (x,y);
            while let Some(&p) = parent.get(&cur) {
                path.push(cur);
                cur = p;
            }
            path.reverse();
            return Some(path);
        }

        if !came.insert((x,y)) { continue; }
        for (nx,ny) in neighbors(x,y, maze) {
            let tentative = g + 1;
            if tentative < *gscore.get(&(nx,ny)).unwrap_or(&i32::MAX) {
                parent.insert((nx,ny), (x,y));
                gscore.insert((nx,ny), tentative);
                let h = manhattan(nx, ny, gx, gy);
                open.push(Node { f: tentative + h, g: tentative, x: nx, y: ny });
            }
        }
    }
    None
}

pub fn update_enemy(
    e: &mut Enemy,
    maze: &Maze,
    player_pos: Vector2,
    block_size: usize,
    dt: f32,
) {
    // Replanifica a ~10Hz
    e.replan_accum += dt;
    if e.replan_accum >= 0.1 {
        e.replan_accum = 0.0;
        let sc = world_to_cell(e.pos.x, e.pos.y, block_size);
        let pc = world_to_cell(player_pos.x, player_pos.y, block_size);
        if e.path.is_empty() || e.path.last().copied() != Some(pc) {
            if let Some(mut path) = astar(maze, sc, pc) {
                if !path.is_empty() && path[0] == sc { path.remove(0); }
                e.path = path;
            } else {
                e.path.clear();
            }
        }
    }

    if let Some(&(nx,ny)) = e.path.first() {
        let target = cell_center(nx, ny, block_size);
        let to = (target - e.pos);
        let dist = to.length();
        let step = e.speed * dt;

        if dist <= step {
            e.pos = target;
            e.path.remove(0);
        } else {
            let dir = to / dist;
            e.pos += dir * step;
        }
    }
}

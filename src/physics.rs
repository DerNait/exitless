// physics.rs
use raylib::prelude::Vector2;
use crate::maze::Maze;

/// ¿Es sólida la celda? (mismo set que tu DDA)
#[inline]
fn is_solid(c: char) -> bool {
    matches!(c, '+' | '-' | '|' | '#' | 'g' | 'Y' | 'B' | 'R' | 'G')
}

/// Empuja un punto (jugador) fuera del AABB de una celda sólida.
#[inline]
fn push_out_of_cell(
    pos: &mut Vector2,
    r: f32,
    cell_i: i32,
    cell_j: i32,
    bs: i32,
) {
    let x0 = (cell_i * bs) as f32;
    let y0 = (cell_j * bs) as f32;
    let x1 = x0 + bs as f32;
    let y1 = y0 + bs as f32;

    // Punto más cercano del AABB al centro del jugador
    let cx = pos.x.clamp(x0, x1);
    let cy = pos.y.clamp(y0, y1);

    let dx = pos.x - cx;
    let dy = pos.y - cy;
    let d2 = dx*dx + dy*dy;

    // Si el centro está dentro del AABB (d2==0) empujamos por eje mínimo.
    if d2 == 0.0 {
        // Distancias a lados
        let left   = (pos.x - x0).abs();
        let right  = (x1 - pos.x).abs();
        let top    = (pos.y - y0).abs();
        let bottom = (y1 - pos.y).abs();

        let m = left.min(right).min(top).min(bottom);
        if m == left     { pos.x = x0 - r; return; }
        if m == right    { pos.x = x1 + r; return; }
        if m == top      { pos.y = y0 - r; return; }
        /* m == bottom */ pos.y = y1 + r;
        return;
    }

    let d = d2.sqrt();

    // Si el círculo penetra el AABB (distancia al borde < r), empujamos
    if d < r {
        let nx = dx / d;
        let ny = dy / d;
        let push = r - d;
        pos.x += nx * push;
        pos.y += ny * push;
    }
}

/// Empuja al jugador fuera de cualquier pared alrededor.
/// `iterations` ayuda a resolver esquinas en múltiples pasos.
pub fn resolve_player_collisions(
    pos: &mut Vector2,
    radius: f32,
    maze: &Maze,
    block_size: usize,
    iterations: usize
) {
    let bs = block_size as i32;

    for _ in 0..iterations {
        let ci = (pos.x / bs as f32).floor() as i32;
        let cj = (pos.y / bs as f32).floor() as i32;

        // Revisamos un vecindario 3x3
        for dj in -1..=1 {
            for di in -1..=1 {
                let ni = ci + di;
                let nj = cj + dj;
                if nj < 0 || ni < 0 { continue; }
                let uj = nj as usize;
                let ui = ni as usize;
                if uj >= maze.len() { continue; }
                if ui >= maze[uj].len() { continue; }

                if is_solid(maze[uj][ui]) {
                    push_out_of_cell(pos, radius, ni, nj, bs);
                }
            }
        }
    }
}

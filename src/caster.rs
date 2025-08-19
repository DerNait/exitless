use raylib::color::Color;
use crate::framebuffer::Framebuffer;
use crate::player::Player;
use crate::maze::Maze;

/// Qué golpeó el rayo y a qué distancia (en píxeles).
pub struct Intersect {
    pub distance: f32,
    pub impact:   char,
}

/// Lanza un rayo desde el jugador en ángulo `a`.
/// Si `draw_line` es true, va pintando los puntos (útil para el mapa 2D).
pub fn cast_ray(
    framebuffer: &mut Framebuffer,
    maze: &Maze,
    player: &Player,
    block_size: usize,
    a: f32,
    draw_line: bool,
) -> Intersect {
    let mut d = 0.0_f32;
    if draw_line {
        framebuffer.set_current_color(Color::WHITESMOKE);
    }

    loop {
        let x = (player.pos.x + d * a.cos()) as i32;
        let y = (player.pos.y + d * a.sin()) as i32;

        // convertir a celdas
        let i = if x < 0 { 0 } else { (x as usize) / block_size };
        let j = if y < 0 { 0 } else { (y as usize) / block_size };

        if j >= maze.len() || i >= maze[j].len() {
            return Intersect { distance: d, impact: ' ' }; // fuera del mapa
        }

        if maze[j][i] != ' ' {
            return Intersect { distance: d, impact: maze[j][i] };
        }

        if draw_line {
            framebuffer.set_pixel(x, y);
        }

        d += 1.0; // paso en píxeles
        if d > 20_000.0 { return Intersect { distance: d, impact: ' ' }; }
    }
}

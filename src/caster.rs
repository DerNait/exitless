use raylib::color::Color;
use crate::framebuffer::Framebuffer;
use crate::player::Player;
use crate::maze::Maze;

/// Qué golpeó el rayo y a qué distancia (en píxeles) + parte fraccional para UV.
pub struct Intersect {
    pub distance: f32,
    pub impact:   char,
    /// fracción 0..1 a lo largo de la cara golpeada (para calcular tx)
    pub hit_frac: f32,
}

/// Lanza un rayo desde el jugador en ángulo `a`.
/// Si `draw_line` es true, dibuja puntos espaciados a lo largo del rayo (solo para vista 2D).
pub fn cast_ray(
    framebuffer: &mut Framebuffer,
    maze: &Maze,
    player: &Player,
    block_size: usize,
    a: f32,
    draw_line: bool,
) -> Intersect {
    let dir_x = a.cos();
    let dir_y = a.sin();
    let bs = block_size as f32;

    // mapa (celda) donde empieza el jugador
    let mut map_x = (player.pos.x / bs).floor() as i32;
    let mut map_y = (player.pos.y / bs).floor() as i32;

    // distancias para avanzar de borde a borde de celda
    let delta_dist_x = if dir_x.abs() < 1e-6 { f32::INFINITY } else { (bs / dir_x.abs()) };
    let delta_dist_y = if dir_y.abs() < 1e-6 { f32::INFINITY } else { (bs / dir_y.abs()) };

    // pasos y distancias iniciales hasta el primer borde
    let (step_x, mut side_dist_x) = if dir_x < 0.0 {
        let dist = ((player.pos.x - (map_x as f32 * bs)) / dir_x.abs()).abs();
        (-1, dist)
    } else {
        let dist = ((((map_x + 1) as f32 * bs) - player.pos.x) / dir_x.abs()).abs();
        (1, dist)
    };

    let (step_y, mut side_dist_y) = if dir_y < 0.0 {
        let dist = ((player.pos.y - (map_y as f32 * bs)) / dir_y.abs()).abs();
        (-1, dist)
    } else {
        let dist = ((((map_y + 1) as f32 * bs) - player.pos.y) / dir_y.abs()).abs();
        (1, dist)
    };

    let mut hit = false;
    let mut side = 0; // 0 = cruce vertical (pared "NS"), 1 = horizontal (pared "EW")

    // DDA: saltar de borde a borde de celda
    while !hit {
        if side_dist_x < side_dist_y {
            side_dist_x += delta_dist_x;
            map_x += step_x;
            side = 0;
        } else {
            side_dist_y += delta_dist_y;
            map_y += step_y;
            side = 1;
        }

        if map_y < 0 || map_y as usize >= maze.len() { break; }
        if map_x < 0 || map_x as usize >= maze[map_y as usize].len() { break; }

        if maze[map_y as usize][map_x as usize] != ' ' {
            hit = true;
        }
    }

    // Distancia al primer muro (corrección a la última suma)
    let dist = if hit {
        let raw = if side == 0 { (side_dist_x - delta_dist_x) } else { (side_dist_y - delta_dist_y) };
        raw.max(1e-4)
    } else {
        20_000.0
    };

    // Coordenada del punto de impacto para hit_frac
    let xf = player.pos.x + dist * dir_x;
    let yf = player.pos.y + dist * dir_y;
    let local_x = (xf % bs + bs) % bs;
    let local_y = (yf % bs + bs) % bs;
    let hit_frac = if side == 0 { (local_y / bs).fract() } else { (local_x / bs).fract() };

    let impact = if hit {
        maze[map_y as usize][map_x as usize]
    } else {
        ' '
    };

    // Dibujo de rayo en 2D (para depurar/vista 2D), sampleado cada ~4px para no matar FPS
    if draw_line {
        let step = 4.0_f32;
        let mut t = 0.0_f32;
        while t <= dist {
            let x = (player.pos.x + dir_x * t) as i32;
            let y = (player.pos.y + dir_y * t) as i32;
            framebuffer.set_current_color(Color::WHITESMOKE);
            framebuffer.set_pixel(x, y);
            t += step;
        }
    }

    Intersect { distance: dist, impact, hit_frac }
}

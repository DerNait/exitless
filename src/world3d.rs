use raylib::prelude::*;
use crate::framebuffer::Framebuffer;
use crate::maze::Maze;
use crate::player::Player;
use crate::caster::{cast_ray};

fn wall_color(c: char) -> Color {
    match c {
        '+' => Color::LIGHTGRAY,
        '-' => Color::GRAY,
        '|' => Color::DARKGRAY,
        'g' => Color::RED,
        _   => Color::WHITE,
    }
}

/// Render tipo Wolfenstein: una columna por píxel de ancho de pantalla.
pub fn render_world(
    framebuffer: &mut Framebuffer,
    maze: &Maze,
    player: &Player,
    block_size: usize,
) {
    let w = framebuffer.width as i32;
    let h = framebuffer.height as i32;
    let hw = (w as f32) * 0.5;
    let hh = (h as f32) * 0.5;

    // distancia al plano de proyección (clásico)
    let dist_to_plane = hw / (player.fov * 0.5).tan();

    // cielo y piso planos (opcional)
    for y in 0..h {
        let is_floor = y as f32 >= hh;
        framebuffer.set_current_color(if is_floor { Color::DARKBROWN } else { Color::DARKBLUE });
        for x in 0..w {
            framebuffer.set_pixel(x, y);
        }
    }

    for i in 0..w {
        // ángulo del rayo i
        let t = i as f32 / (w as f32);
        let ray_a = player.a - (player.fov * 0.5) + (player.fov * t);

        // intersect sin dibujar línea en el mapa 2D
        let inter = cast_ray(framebuffer, maze, player, block_size, ray_a, false);

        // corrección de ojo de pez
        let delta = ray_a - player.a;
        let dist = inter.distance * delta.cos().max(1e-4);

        // altura proyectada (cuanto más cerca, más alto)
        // fórmula: (tamaño_real * dist_plano) / distancia
        let wall_real = block_size as f32; // alto "real" del bloque
        let stake_height = (wall_real * dist_to_plane) / dist;

        // coordenadas top/bottom centradas (clamp)
        let mut top = (hh - stake_height * 0.5) as i32;
        let mut bot = (hh + stake_height * 0.5) as i32;
        if top < 0 { top = 0; }
        if bot > h - 1 { bot = h - 1; }

        // sombreado simple: más oscuro si lejos
        let mut col = wall_color(inter.impact);
        let fade = (1.0 / (1.0 + dist * 0.003)).clamp(0.2, 1.0);
        col.r = (col.r as f32 * fade) as u8;
        col.g = (col.g as f32 * fade) as u8;
        col.b = (col.b as f32 * fade) as u8;
        framebuffer.set_current_color(col);

        // dibujar columna i
        for y in top..=bot {
            framebuffer.set_pixel(i, y);
        }
    }
}

use raylib::color::Color;
use crate::framebuffer::Framebuffer;
use crate::maze::Maze;
use crate::player::Player;
use crate::enemy::Enemy;
use crate::caster::{cast_ray, Intersect};

/// Colores configurables del minimapa (RGBA en u8).
#[derive(Clone, Copy)]
pub struct MinimapColors {
    pub wall:     (u8,u8,u8,u8),
    pub empty:    (u8,u8,u8,u8),
    pub goal:     (u8,u8,u8,u8),
    pub player:   (u8,u8,u8,u8),
    pub enemy:    (u8,u8,u8,u8),
    pub dir_line: (u8,u8,u8,u8),
    pub fov_ray:  (u8,u8,u8,u8),
    pub frame:    (u8,u8,u8,u8),
}

impl Default for MinimapColors {
    fn default() -> Self {
        Self {
            wall:     (182, 180, 97,  255),
            empty:    (150, 142, 59,  255),
            goal:     (200, 60,  60,  255),
            player:   (34,  190, 34,  255),
            enemy:    (200, 40,  40,  255),
            dir_line: (180, 255, 180, 255),
            fov_ray:  (110, 160, 255, 160), // semitransparente
            frame:    (0,   0,   0,   160), // borde/fondo minimapa
        }
    }
}

pub fn draw_cell(
    framebuffer: &mut Framebuffer,
    xo: usize,
    yo: usize,
    block_size: usize,
    cell: char,
) {
    let color = match cell {
        '+' | '-' | '|' => Color::DARKGRAY,  // paredes
        'p' => Color::GREEN,                  // player start
        'g' => Color::RED,                    // goal
        'e' => Color::YELLOW,
        _   => Color::BLANK,                  // espacios
    };
    framebuffer.set_current_color(color);

    for y in 0..block_size {
        for x in 0..block_size {
            framebuffer.set_pixel((xo + x) as i32, (yo + y) as i32);
        }
    }
}

/// Render 2D pantalla completa (usado para debug/legacy)
pub fn render_maze(
    framebuffer: &mut Framebuffer,
    maze: &Maze,
    block_size: usize,
) {
    for (row_index, row) in maze.iter().enumerate() {
        for (col_index, cell) in row.iter().enumerate() {
            let xo = col_index * block_size;
            let yo = row_index * block_size;
            draw_cell(framebuffer, xo, yo, block_size, *cell);
        }
    }
}

/// Render de MINIMAPA dentro de un rectángulo (x,y,w,h).
/// Dibuja paredes/espacios, jugador, enemigos, línea de dirección y **rayos de visión**.
pub fn render_minimap(
    fb: &mut Framebuffer,
    maze: &Maze,
    player: &Player,
    enemies: &[Enemy],
    block_size: usize,
    x: i32, y: i32, w: i32, h: i32,
    style: &MinimapColors,
) {
    if w <= 0 || h <= 0 { return; }
    let rows = maze.len() as i32;
    if rows == 0 { return; }
    let cols = maze[0].len() as i32;

    // Tamaño de celda destino (px)
    let csx = (w as f32 / cols as f32).max(1.0) as i32;
    let csy = (h as f32 / rows as f32).max(1.0) as i32;

    // Fondo/borde del minimapa
    fill_rect(fb, x - 2, y - 2, w + 4, h + 4, style.frame);

    // Dibuja celdas
    for j in 0..rows {
        for i in 0..cols {
            let cell = maze[j as usize][i as usize];
            let (r,g,b,a) = match cell {
                '+' | '-' | '|' | '#' => style.wall,
                'g' => style.goal,
                _    => style.empty,
            };
            let dx = x + i * csx;
            let dy = y + j * csy;
            for yy in 0..csy {
                for xx in 0..csx {
                    fb.put_pixel_rgba(dx + xx, dy + yy, r, g, b, a);
                }
            }
        }
    }

    // Player en minimapa
    let px_cells = player.pos.x / block_size as f32;
    let py_cells = player.pos.y / block_size as f32;
    let pxi = (x as f32 + px_cells * csx as f32) as i32;
    let pyi = (y as f32 + py_cells * csy as f32) as i32;
    draw_disc(fb, pxi, pyi, (csx.min(csy) / 3).max(2), style.player);

    // Rayos de visión (sampleado moderado para rendimiento)
    let n_rays = (w / 8).clamp(24, 96) as usize; // de 24 a 96 rayos según ancho
    for k in 0..n_rays {
        let t = if n_rays > 1 { k as f32 / (n_rays - 1) as f32 } else { 0.5 };
        let ray_a = player.a - (player.fov * 0.5) + (player.fov * t);
        let inter: Intersect = cast_ray(
            fb,             // no dibuja en 3D; draw_line=false
            maze,
            player,
            block_size,
            ray_a,
            false,
        );

        let dir_x = ray_a.cos();
        let dir_y = ray_a.sin();
        let hit_world_x = player.pos.x + inter.distance * dir_x;
        let hit_world_y = player.pos.y + inter.distance * dir_y;

        // Convertir a coords de celdas → píxeles en el rectángulo del minimapa
        let hx_cells = hit_world_x / block_size as f32;
        let hy_cells = hit_world_y / block_size as f32;
        let hxi = (x as f32 + hx_cells * csx as f32) as i32;
        let hyi = (y as f32 + hy_cells * csy as f32) as i32;

        draw_line(fb, pxi, pyi, hxi, hyi, style.fov_ray);
    }

    // Enemigos
    for e in enemies {
        let ex = e.pos.x / block_size as f32;
        let ey = e.pos.y / block_size as f32;
        let exi = (x as f32 + ex * csx as f32) as i32;
        let eyi = (y as f32 + ey * csy as f32) as i32;
        draw_disc(fb, exi, eyi, (csx.min(csy) / 3).max(2), style.enemy);
    }

    // Línea de dirección del player
    let dir_len = (csx.min(csy) * 2).max(8) as f32;
    let dx = player.a.cos() * dir_len;
    let dy = player.a.sin() * dir_len;
    draw_line(fb, pxi, pyi, (pxi as f32 + dx) as i32, (pyi as f32 + dy) as i32, style.dir_line);
}

// Utilidades

fn draw_disc(fb: &mut Framebuffer, cx: i32, cy: i32, r: i32, color: (u8,u8,u8,u8)) {
    let (rr,gg,bb,aa) = color;
    for y in -r..=r {
        for x in -r..=r {
            if x*x + y*y <= r*r {
                fb.put_pixel_rgba(cx + x, cy + y, rr, gg, bb, aa);
            }
        }
    }
}

fn draw_line(fb: &mut Framebuffer, x0: i32, y0: i32, x1: i32, y1: i32, color: (u8,u8,u8,u8)) {
    let (rr,gg,bb,aa) = color;
    let mut x0 = x0; let mut y0 = y0;
    let dx = (x1 - x0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let dy = -(y1 - y0).abs();
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;
    loop {
        fb.put_pixel_rgba(x0, y0, rr, gg, bb, aa);
        if x0 == x1 && y0 == y1 { break; }
        let e2 = 2*err;
        if e2 >= dy { err += dy; x0 += sx; }
        if e2 <= dx { err += dx; y0 += sy; }
    }
}

fn fill_rect(fb: &mut Framebuffer, dx: i32, dy: i32, dw: i32, dh: i32, color: (u8,u8,u8,u8)) {
    let max_x = (dx + dw).min(fb.width);
    let max_y = (dy + dh).min(fb.height);
    for yy in dy.max(0)..max_y {
        for xx in dx.max(0)..max_x {
            fb.put_pixel_rgba(xx, yy, color.0, color.1, color.2, color.3);
        }
    }
}

use raylib::color::Color;
use crate::framebuffer::Framebuffer;
use crate::maze::Maze;
use crate::player::Player;
use crate::enemy::Enemy;

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

/// Render 2D pantalla completa (modo debug / legacy)
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

/// Render de MINIMAPA dentro de un rectángulo (x,y,w,h) del framebuffer.
/// Escala el grid para caber en el rectángulo; dibuja paredes, player y enemigos.
pub fn render_minimap(
    fb: &mut Framebuffer,
    maze: &Maze,
    player: &Player,
    enemies: &[Enemy],
    block_size: usize,
    x: i32, y: i32, w: i32, h: i32
) {
    if w <= 0 || h <= 0 { return; }
    let rows = maze.len() as i32;
    if rows == 0 { return; }
    let cols = maze[0].len() as i32;

    // Tamaño de celda destino (px)
    let csx = (w as f32 / cols as f32).max(1.0) as i32;
    let csy = (h as f32 / rows as f32).max(1.0) as i32;

    // Dibuja celdas
    for j in 0..rows {
        for i in 0..cols {
            let cell = maze[j as usize][i as usize];
            let (r,g,b,a) = match cell {
                '+' | '-' | '|' | '#' => (90, 90, 90, 255),
                'g' => (200, 60, 60, 255),
                _    => (20, 20, 20, 255),
            };
            // fill rect de la celda
            let dx = x + i * csx;
            let dy = y + j * csy;
            for yy in 0..csy {
                for xx in 0..csx {
                    fb.put_pixel_rgba(dx + xx, dy + yy, r, g, b, a);
                }
            }
        }
    }

    // Player en minimapa (punto verde)
    let px = (player.pos.x / block_size as f32) as f32;
    let py = (player.pos.y / block_size as f32) as f32;
    let pxi = (x as f32 + px * csx as f32) as i32;
    let pyi = (y as f32 + py * csy as f32) as i32;
    draw_disc(fb, pxi, pyi, (csx.min(csy) / 3).max(2), (34, 190, 34, 255));

    // Enemigos (puntos rojos)
    for e in enemies {
        let ex = (e.pos.x / block_size as f32) as f32;
        let ey = (e.pos.y / block_size as f32) as f32;
        let exi = (x as f32 + ex * csx as f32) as i32;
        let eyi = (y as f32 + ey * csy as f32) as i32;
        draw_disc(fb, exi, eyi, (csx.min(csy) / 3).max(2), (200, 40, 40, 255));
    }

    // Línea de dirección del player
    let dir_len = (csx.min(csy) * 2).max(8) as f32;
    let dx = player.a.cos() * dir_len;
    let dy = player.a.sin() * dir_len;
    draw_line(fb, pxi, pyi, (pxi as f32 + dx) as i32, (pyi as f32 + dy) as i32, (180, 255, 180, 255));
}

// Utilidades simples para dibujar primitivos
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

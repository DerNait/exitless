use raylib::color::Color;
use crate::framebuffer::Framebuffer;
use crate::maze::Maze;
use crate::player::Player;
use crate::enemy::Enemy;
use crate::sprites::Sprite; // ⬅️ para dibujar llaves
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

    // ⬇️ NUEVO: colores para cada llave
    pub key_y:    (u8,u8,u8,u8),
    pub key_b:    (u8,u8,u8,u8),
    pub key_r:    (u8,u8,u8,u8),
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

            key_y:    (255, 215, 0,   255), // amarillo
            key_b:    (60,  130, 255, 255), // azul
            key_r:    (235, 60,  60,  255), // rojo
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
        'g' => Color::RED,                    // goal (legacy)
        'e' => Color::YELLOW,
        // puertas también podrían mostrarse aquí si usas render_maze
        'Y' | 'B' | 'R' | 'G' => Color::DARKGRAY,
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

/// Mini-mapa “zoom cam” suave: ventana de `view_w_cells`×`view_h_cells`
/// centrada en el jugador con **offset fraccionario** y clamped a bordes.
/// Dibuja paredes/espacios por *pixel*, jugador, enemigos, rayos y **llaves**.
pub fn render_minimap_zoomed(
    fb: &mut Framebuffer,
    maze: &Maze,
    player: &Player,
    enemies: &[Enemy],
    keys_sprites: &[Sprite],      // ⬅️ NUEVO: llaves
    block_size: usize,
    x: i32, y: i32, w: i32, h: i32,
    view_w_cells: i32,
    view_h_cells: i32,
    style: &MinimapColors,
) {
    if w <= 0 || h <= 0 { return; }
    let rows = maze.len() as i32;
    if rows == 0 { return; }
    let cols = maze[0].len() as i32;

    // Tamaño de ventana en celdas (clamp a mapa)
    let vw = view_w_cells.clamp(1, cols);
    let vh = view_h_cells.clamp(1, rows);

    // Centro del jugador en celdas (float)
    let pcx = player.pos.x / block_size as f32;
    let pcy = player.pos.y / block_size as f32;

    // Origen fraccional de la ventana (clamp a bordes)
    let mut start_i_f = pcx - (vw as f32) * 0.5;
    let mut start_j_f = pcy - (vh as f32) * 0.5;
    let max_start_i_f = (cols - vw).max(0) as f32;
    let max_start_j_f = (rows - vh).max(0) as f32;
    if start_i_f < 0.0 { start_i_f = 0.0; }
    if start_j_f < 0.0 { start_j_f = 0.0; }
    if start_i_f > max_start_i_f { start_i_f = max_start_i_f; }
    if start_j_f > max_start_j_f { start_j_f = max_start_j_f; }

    // Fondo/borde del minimapa
    fill_rect(fb, x - 2, y - 2, w + 4, h + 4, style.frame);

    // --- Dibujo por pixel (nearest) sin dejar bordes ---
    for yy in 0..h {
        // v en [0,1)
        let v = (yy as f32 + 0.5) / (h as f32);
        let sy = start_j_f + v * (vh as f32);
        // celda Y
        let mut j = sy.floor() as i32;
        if j < 0 { j = 0; }
        if j >= rows { j = rows - 1; }

        for xx in 0..w {
            let u = (xx as f32 + 0.5) / (w as f32);
            let sx = start_i_f + u * (vw as f32);
            // celda X
            let mut i = sx.floor() as i32;
            if i < 0 { i = 0; }
            if i >= cols { i = cols - 1; }

            let cell = maze[j as usize][i as usize];
            let (r,g,b,a) = match cell {
                // ⬅️ Puertas como paredes
                '+' | '-' | '|' | '#' | 'Y' | 'B' | 'R' | 'G' => style.wall,
                'g' => style.goal, // (legacy)
                _    => style.empty,
            };
            fb.put_pixel_rgba(x + xx, y + yy, r, g, b, a);
        }
    }

    // Map helpers: de coords de celda (float) -> pixel en el rectángulo
    let to_px = |cx: f32, cy: f32| -> (i32,i32) {
        let u = ((cx - start_i_f) / (vw as f32)).clamp(0.0, 1.0);
        let v = ((cy - start_j_f) / (vh as f32)).clamp(0.0, 1.0);
        let px = x + (u * (w as f32 - 1.0)).round() as i32;
        let py = y + (v * (h as f32 - 1.0)).round() as i32;
        (px, py)
    };

    // Jugador
    let (pxi, pyi) = to_px(pcx, pcy);
    let cell_px = ((w as f32 / vw as f32).min(h as f32 / vh as f32)) as i32;
    let pr = (cell_px / 3).max(2);
    draw_disc(fb, pxi, pyi, pr, style.player);

    // Rayos de visión (clamp a la ventana)
    let n_rays = (w / 8).clamp(24, 96) as usize;
    for k in 0..n_rays {
        let t = if n_rays > 1 { k as f32 / (n_rays - 1) as f32 } else { 0.5 };
        let ray_a = player.a - (player.fov * 0.5) + (player.fov * t);
        let inter: Intersect = cast_ray(fb, maze, player, block_size, ray_a, false);

        let dir_x = ray_a.cos();
        let dir_y = ray_a.sin();
        let hx_cells = (player.pos.x + inter.distance * dir_x) / block_size as f32;
        let hy_cells = (player.pos.y + inter.distance * dir_y) / block_size as f32;

        let mut hx = hx_cells;
        let mut hy = hy_cells;
        // clamp a la ventana para que la línea no se salga
        if hx < start_i_f { hx = start_i_f; }
        if hy < start_j_f { hy = start_j_f; }
        if hx > start_i_f + vw as f32 { hx = start_i_f + vw as f32; }
        if hy > start_j_f + vh as f32 { hy = start_j_f + vh as f32; }

        let (hxi, hyi) = to_px(hx, hy);
        draw_line(fb, pxi, pyi, hxi, hyi, style.fov_ray);
    }

    // Enemigos (solo los que caen dentro de la ventana)
    for e in enemies {
        let ex = e.pos.x / block_size as f32;
        let ey = e.pos.y / block_size as f32;
        if ex >= start_i_f && ex <= start_i_f + vw as f32 &&
           ey >= start_j_f && ey <= start_j_f + vh as f32 {
            let (exi, eyi) = to_px(ex, ey);
            draw_disc(fb, exi, eyi, pr, style.enemy);
        }
    }

    // ⬇️ NUEVO: Llaves (discos). Usamos su `Sprite.pos` actual.
    for s in keys_sprites {
        let kx = s.pos.x / block_size as f32;
        let ky = s.pos.y / block_size as f32;
        if kx >= start_i_f && kx <= start_i_f + vw as f32 &&
           ky >= start_j_f && ky <= start_j_f + vh as f32 {
            let (kxi, kyi) = to_px(kx, ky);
            let color = match s.tex {
                '1' => style.key_y, // amarilla
                '2' => style.key_b, // azul
                '3' => style.key_r, // roja
                _   => style.empty,
            };
            // un pelín más pequeño que el player
            let kr = (pr * 2 / 3).max(2);
            draw_disc(fb, kxi, kyi, kr, color);
        }
    }

    // Dirección del player
    let dir_len = (cell_px * 2).max(8) as f32;
    let dx = player.a.cos() * dir_len;
    let dy = player.a.sin() * dir_len;
    let (dxi, dyi) = to_px(pcx + dx / (block_size as f32), pcy + dy / (block_size as f32));
    draw_line(fb, pxi, pyi, dxi, dyi, style.dir_line);
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

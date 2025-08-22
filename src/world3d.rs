use raylib::prelude::*;
use crate::framebuffer::Framebuffer;
use crate::maze::Maze;
use crate::player::Player;
use crate::caster::cast_ray;
use crate::textures::TextureManager;
use crate::sprites::{Sprite, render_sprites};
use crate::enemy::Enemy;

fn sky_floor_region(fb: &mut Framebuffer, y0: i32, vh: i32) {
    let half = vh / 2;
    let sky   = Color::new(150, 142, 59, 255);
    let floor = Color::new(133, 111, 27, 255);
    for i in 0..half {
        fb.fill_row(y0 + i, sky);
    }
    for i in half..vh {
        fb.fill_row(y0 + i, floor);
    }
}

/// Overlay en pantalla completa (se mantiene por compatibilidad)
pub fn draw_overlay_fullscreen(fb: &mut Framebuffer, tex: &TextureManager, key: char) {
    draw_overlay_viewport(fb, tex, key, 0, 0, fb.width, fb.height);
}

/// ⬅️ Overlay limitado a un rectángulo (viewport 3D)
pub fn draw_overlay_viewport(
    fb: &mut Framebuffer,
    tex: &TextureManager,
    key: char,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
) {
    let (src_w, src_h, data) = tex.tex_view(key);
    if src_w == 0 || src_h == 0 || w <= 0 || h <= 0 { return; }

    let max_x = (x + w).min(fb.width);
    let max_y = (y + h).min(fb.height);
    let start_x = x.max(0);
    let start_y = y.max(0);
    let dst_w = max_x - start_x;
    let dst_h = max_y - start_y;
    if dst_w <= 0 || dst_h <= 0 { return; }

    for yy in 0..dst_h {
        let sy = (((yy as f32) / (h as f32)) * (src_h as f32)).floor() as i32;
        let sy = sy.clamp(0, src_h as i32 - 1);
        for xx in 0..dst_w {
            let sx = (((xx as f32) / (w as f32)) * (src_w as f32)).floor() as i32;
            let sx = sx.clamp(0, src_w as i32 - 1);

            let idx = (((sy as usize) * src_w) + (sx as usize)) * 4;
            let r = data[idx];
            let g = data[idx + 1];
            let b = data[idx + 2];
            let a = data[idx + 3];
            if a > 0 {
                fb.put_pixel_rgba(start_x + xx, start_y + yy, r, g, b, a);
            }
        }
    }
}

pub fn draw_game_over_background(fb: &mut Framebuffer) {
    for y in 0..fb.height {
        fb.fill_row(y, Color::BLACK);
    }
}

pub fn draw_win_background(fb: &mut Framebuffer) {
    for y in 0..fb.height {
        fb.fill_row(y, Color::DARKGREEN);
    }
}

/// Render 3D **en un viewport** (0..w, y0..y0+vh) con sprites y llaves ocluidas por muros.
pub fn render_world_textured(
    fb: &mut Framebuffer,
    maze: &Maze,
    player: &Player,
    block_size: usize,
    tex: &TextureManager,
    sprites: &[Sprite],        // decorativos estáticos (si tienes)
    enemies: &[Enemy],         // enemigos (se convierten a sprites dinámicos)
    keys_sprites: &[Sprite],   // 🔑 llaves animadas
    time_s: f32,
    viewport_y0: i32,
    viewport_h: i32,
) {
    let w = fb.width as i32;
    let h = viewport_h.max(1);
    let y_off = viewport_y0.max(0);

    let hw = w as f32 * 0.5;
    let hh = h as f32 * 0.5;
    let dist_to_plane = hw / (player.fov * 0.5).tan();

    // Fondo cielo/suelo dentro del viewport
    sky_floor_region(fb, y_off, h);

    // z-buffer por columna
    let mut zbuf = vec![f32::INFINITY; w as usize];

    for i in 0..w {
        let t = i as f32 / w as f32;
        let ray_a = player.a - (player.fov * 0.5) + (player.fov * t);
        let inter = cast_ray(fb, maze, player, block_size, ray_a, false);
        let delta = ray_a - player.a;
        let dist = (inter.distance * delta.cos()).max(1e-4);
        zbuf[i as usize] = dist;

        let wall_real = block_size as f32;
        let line_h = ((wall_real * dist_to_plane) / dist).max(1.0);

        let mut draw_start = (hh - line_h * 0.5).floor() as i32 + y_off;
        let mut draw_end   = (hh + line_h * 0.5).ceil()  as i32 + y_off;

        let y_min = y_off;
        let y_max = y_off + h - 1;
        if draw_start < y_min { draw_start = y_min; }
        if draw_end   > y_max { draw_end   = y_max; }

        let ch = if inter.impact == ' ' { '#' } else { inter.impact };
        let (tw, th, tdata) = tex.tex_view(ch);

        let mut tx = (inter.hit_frac * tw as f32).floor() as i32;
        if tx < 0 { tx = 0; }
        if tx >= tw as i32 { tx = tw as i32 - 1; }

        let step = th as f32 / line_h;
        let start_tex_pos = ((draw_start as f32 - ((y_off as f32) + hh - line_h * 0.5)) * step)
            .max(0.0);
        let mut tex_pos = start_tex_pos;

        let shade = (1.0 / (1.0 + dist * 0.001)).clamp(0.7, 1.0);

        for y in draw_start..=draw_end {
            let mut ty = tex_pos as i32;
            if ty < 0 { ty = 0; }
            if ty >= th as i32 { ty = th as i32 - 1; }
            tex_pos += step;

            let idx = ((ty as usize * tw) + tx as usize) * 4;
            let (r, g, b) = (tdata[idx], tdata[idx + 1], tdata[idx + 2]);

            let rr = (r as f32 * shade) as u8;
            let gg = (g as f32 * shade) as u8;
            let bb = (b as f32 * shade) as u8;

            fb.put_pixel_rgba(i, y, rr, gg, bb, 255);
        }
    }

    // Sprites decorativos estáticos (si los usas)
    render_sprites(fb, player, sprites, tex, &zbuf, block_size, time_s, y_off, h);

    // 🔑 Llaves animadas, ocluidas por muros (usa el mismo zbuf)
    render_sprites(fb, player, keys_sprites, tex, &zbuf, block_size, time_s, y_off, h);

    // Enemigos como sprites animados dinámicos (igual que antes)
    use crate::sprites::Sprite as DynSprite;
    let mut dyn_sprites: Vec<DynSprite> = Vec::new();
    for e in enemies {
        dyn_sprites.push(DynSprite {
            pos: e.pos,
            tex: 'e',
            scale: 1.0,
            frames: tex.sheet_frames('e'),
            fps: 8.0,
            phase: 0,
        });
    }
    render_sprites(fb, player, &dyn_sprites, tex, &zbuf, block_size, time_s, y_off, h);
}

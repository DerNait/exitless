use raylib::prelude::*;
use crate::framebuffer::Framebuffer;
use crate::maze::Maze;
use crate::player::Player;
use crate::caster::cast_ray;
use crate::textures::TextureManager;
use crate::sprites::{Sprite, render_sprites};
use crate::enemy::Enemy;

fn sky_floor(fb: &mut Framebuffer) {
    let h = fb.height;
    let half = h / 2;
    let sky   = Color::new(150, 142, 59, 255);
    let floor = Color::new(133, 111, 27, 255);
    for y in 0..half { fb.fill_row(y, sky); }
    for y in half..h { fb.fill_row(y, floor); }
}

/// Dibuja una imagen del TextureManager a pantalla completa (nearest).
pub fn draw_overlay_fullscreen(fb: &mut Framebuffer, tex: &TextureManager, key: char) {
    let (src_w, src_h, data) = tex.tex_view(key);
    if src_w == 0 || src_h == 0 { return; }

    let dst_w = fb.width as i32;
    let dst_h = fb.height as i32;

    for y in 0..dst_h {
        // fila fuente (nearest)
        let sy = ((y as f32 / dst_h as f32) * src_h as f32).floor() as i32;
        let sy = sy.clamp(0, src_h as i32 - 1);

        for x in 0..dst_w {
            let sx = ((x as f32 / dst_w as f32) * src_w as f32).floor() as i32;
            let sx = sx.clamp(0, src_w as i32 - 1);

            let idx = (((sy as usize) * src_w) + (sx as usize)) * 4;
            let r = data[idx];
            let g = data[idx + 1];
            let b = data[idx + 2];
            let a = data[idx + 3];
            // Si tu jumpscare tiene alpha, respétalo; si no, a será 255
            if a > 0 {
                fb.put_pixel_rgba(x, y, r, g, b, a);
            }
        }
    }
}

/// Llena la pantalla con negro (el texto "GAME OVER" se dibuja en main con Raylib)
pub fn draw_game_over_background(fb: &mut Framebuffer) {
    for y in 0..fb.height {
        fb.fill_row(y, Color::BLACK);
    }
}

pub fn render_world_textured(
    fb: &mut Framebuffer,
    maze: &Maze,
    player: &Player,
    block_size: usize,
    tex: &TextureManager,
    sprites: &[Sprite],
    enemies: &[Enemy],    // 👈 añadido
    time_s: f32,
) {
    let w = fb.width as i32;
    let h = fb.height as i32;
    let hw = w as f32 * 0.5;
    let hh = h as f32 * 0.5;
    let dist_to_plane = hw / (player.fov * 0.5).tan();

    sky_floor(fb);

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

        let mut draw_start = (hh - line_h * 0.5).floor() as i32;
        let mut draw_end   = (hh + line_h * 0.5).ceil()  as i32;
        if draw_start < 0 { draw_start = 0; }
        if draw_end >= h  { draw_end = h - 1; }

        let ch = if inter.impact == ' ' { '#' } else { inter.impact };
        let (tw, th, tdata) = tex.tex_view(ch);

        let mut tx = (inter.hit_frac * tw as f32).floor() as i32;
        if tx < 0 { tx = 0; }
        if tx >= tw as i32 { tx = tw as i32 - 1; }

        let step = th as f32 / line_h;
        let mut tex_pos = (draw_start as f32 - (hh - line_h * 0.5)) * step;

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

    // Sprites decorativos
    render_sprites(fb, player, sprites, tex, &zbuf, block_size, time_s);

    // Enemigos como sprites animados dinámicos
    use crate::sprites::Sprite;
    let mut dyn_sprites: Vec<Sprite> = Vec::new();
    for e in enemies {
        dyn_sprites.push(Sprite {
            pos: e.pos,
            tex: 'e',
            scale: 1.0,
            frames: tex.sheet_frames('e'),
            fps: 8.0,
            phase: 0,
        });
    }
    render_sprites(fb, player, &dyn_sprites, tex, &zbuf, block_size, time_s);
}

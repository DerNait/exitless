use raylib::prelude::*;
use crate::framebuffer::Framebuffer;
use crate::maze::Maze;
use crate::player::Player;
use crate::caster::cast_ray;
use crate::textures::TextureManager;

fn sky_floor(fb: &mut Framebuffer) {
    let h = fb.height;
    let half = h / 2;
    let sky   = Color::new(150, 142, 59, 255);
    let floor = Color::new(161, 154, 73, 255);
    for y in 0..half      { fb.fill_row(y, sky); }
    for y in half..h      { fb.fill_row(y, floor); }
}

pub fn render_world_textured(
    fb: &mut Framebuffer,
    maze: &Maze,
    player: &Player,
    block_size: usize,
    tex: &TextureManager,
) {
    let w = fb.width as i32;
    let h = fb.height as i32;
    let hw = w as f32 * 0.5;
    let hh = h as f32 * 0.5;
    let dist_to_plane = hw / (player.fov * 0.5).tan();

    // repinta todo el fondo
    sky_floor(fb);

    for i in 0..w {
        let t = i as f32 / w as f32;
        let ray_a = player.a - (player.fov * 0.5) + (player.fov * t);

        // rayo (DDA)
        let inter = cast_ray(fb, maze, player, block_size, ray_a, false);

        // corrección de ojo de pez
        let delta = ray_a - player.a;
        let dist = (inter.distance * delta.cos()).max(1e-4);

        // alto proyectado
        let wall_real = block_size as f32;
        let line_h = ((wall_real * dist_to_plane) / dist).max(1.0);

        // top/bottom de la columna
        let mut draw_start = (hh - line_h * 0.5).floor() as i32;
        let mut draw_end   = (hh + line_h * 0.5).ceil()  as i32;
        if draw_start < 0 { draw_start = 0; }
        if draw_end >= h  { draw_end = h - 1; }

        // textura y coords
        let ch = if inter.impact == ' ' { '#' } else { inter.impact };
        let (tw, th, tdata) = tex.tex_view(ch);

        let mut tx = (inter.hit_frac * tw as f32).floor() as i32;
        if tx < 0 { tx = 0; }
        if tx >= tw as i32 { tx = tw as i32 - 1; }

        // mapeo vertical
        let step = th as f32 / line_h;
        let mut tex_pos = (draw_start as f32 - (hh - line_h * 0.5)) * step;

        // sombreado constante por columna
        let shade = (1.0 / (1.0 + dist * 0.003)).clamp(0.7, 1.0);

        for y in draw_start..=draw_end {
            let mut ty = tex_pos as i32;
            if ty < 0 { ty = 0; }
            if ty >= th as i32 { ty = th as i32 - 1; }
            tex_pos += step;

            let idx = ((ty as usize * tw) + tx as usize) * 4;
            let (r, g, b) = (tdata[idx], tdata[idx + 1], tdata[idx + 2]);

            // aplica sombreado (float simple; si quieres aún más perf, pasa a fixed-point)
            let rr = (r as f32 * shade) as u8;
            let gg = (g as f32 * shade) as u8;
            let bb = (b as f32 * shade) as u8;

            fb.put_pixel_rgba(i, y, rr, gg, bb, 255);
        }
    }
}

use raylib::prelude::Vector2;
use crate::framebuffer::Framebuffer;
use crate::maze::Maze;
use crate::player::Player;
use crate::textures::TextureManager;

const PI: f32 = std::f32::consts::PI;
const TWO_PI: f32 = std::f32::consts::TAU;
// Si tus PNGs tienen alpha real, puedes quitar el key para ahorrar comparaciones:
const TRANSPARENT_KEY: (u8,u8,u8) = (152,0,136);

#[derive(Clone, Copy, Debug)]
pub struct Sprite {
    pub pos: Vector2,
    pub tex: char,
    pub scale: f32,
    pub frames: usize,
    pub fps: f32,
    pub phase: usize,
}

impl Sprite {
    pub fn new_animated(pos: Vector2, tex: char, scale: f32, frames: usize, fps: f32, phase: usize) -> Self {
        Self { pos, tex, scale, frames: frames.max(1), fps, phase }
    }
}

#[inline]
fn is_key_tex(ch: char) -> bool { ch == '1' || ch == '2' || ch == '3' }

pub fn collect_sprites(maze: &Maze, block_size: usize, tex: &TextureManager) -> Vec<Sprite> {
    let mut v = Vec::new();
    for (j, row) in maze.iter().enumerate() {
        for (i, &c) in row.iter().enumerate() {
            if c == 'e' {
                let x = (i * block_size + block_size / 2) as f32;
                let y = (j * block_size + block_size / 2) as f32;
                let frames = tex.sheet_frames('e');
                v.push(Sprite::new_animated(Vector2::new(x,y), 'e', 1.0, frames, 8.0, (i+j)%frames));
            }
        }
    }
    v
}

/// Recolecta llaves ubicadas en el mapa con caracteres '1','2','3'
/// y devuelve un vector de sprites. Luego puedes limpiar el mapa.
pub fn collect_keys(maze: &Maze, block_size: usize, tex: &TextureManager) -> Vec<Sprite> {
    let mut v = Vec::new();
    for (j, row) in maze.iter().enumerate() {
        for (i, &c) in row.iter().enumerate() {
            if is_key_tex(c) {
                let x = (i * block_size + block_size / 2) as f32;
                let y = (j * block_size + block_size / 2) as f32;
                let frames = tex.sheet_frames(c);
                v.push(Sprite::new_animated(Vector2::new(x,y), c, 1.0, frames, 6.0, 0));
            }
        }
    }
    v
}

#[inline] fn normalize_angle(mut a: f32) -> f32 { while a>PI {a-=TWO_PI;} while a<(-PI) {a+=TWO_PI;} a }

pub fn render_sprites(
    fb: &mut Framebuffer,
    player: &Player,
    sprites: &[Sprite],
    tex: &TextureManager,
    zbuf: &[f32],
    block_size: usize,
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

    // Orden por distancia (lejos -> cerca)
    let mut order: Vec<(usize, f32)> = sprites.iter().enumerate()
        .map(|(idx,s)| { let dx=s.pos.x-player.pos.x; let dy=s.pos.y-player.pos.y; (idx,(dx*dx+dy*dy).sqrt()) })
        .collect();
    order.sort_by(|a,b| b.1.partial_cmp(&a.1).unwrap());

    for (idx, _euclid) in order {
        let s = sprites[idx];
        let dx = s.pos.x - player.pos.x;
        let dy = s.pos.y - player.pos.y;

        let ca = player.a.cos();
        let sa = player.a.sin();

        let mut perp = dx * ca + dy * sa;
        if perp <= 0.0 { continue; }

        let near_plane = 1.0;
        if perp < near_plane { perp = near_plane; }

        let lateral = -dx * sa + dy * ca;

        let screen_x = hw + (lateral * dist_to_plane) / perp;
        let base = block_size as f32 * s.scale;
        let mut sprite_h = ((base * dist_to_plane) / perp).max(1.0);
        let mut sprite_w = sprite_h;

        let left = screen_x - sprite_w * 0.5;
        let top  = (y_off as f32) + hh - sprite_h * 0.5;

        let start_x_raw = left.floor() as i32;
        let end_x_raw   = (left + sprite_w).ceil() as i32 - 1;
        let start_y_raw = top.floor() as i32;
        let end_y_raw   = (top + sprite_h).ceil() as i32 - 1;

        // Clip a viewport
        let y_min = y_off;
        let y_max = y_off + h - 1;

        if end_x_raw < 0 || start_x_raw > (w-1) || end_y_raw < y_min || start_y_raw > y_max {
            continue;
        }

        let mut start_x = start_x_raw.max(0);
        let mut end_x   = end_x_raw.min(w-1);
        let mut start_y = start_y_raw.max(y_min);
        let mut end_y   = end_y_raw.min(y_max);
        if start_x > end_x || start_y > end_y { continue; }

        let anim_fps = s.fps.max(1.0);
        let frame_i = (time_s * anim_fps).floor() as usize;
        let frame = if s.frames>1 { (frame_i + s.phase) % s.frames } else { 0 };

        let (tw, th, x0, y0, fw_us, fh_us, tdata) = tex.sheet_frame_view(s.tex, frame);
        let fw = fw_us as i32;
        let fh = fh_us as i32;

        let step_tx = fw as f32 / sprite_w;
        let step_ty = fh as f32 / sprite_h;

        let mut tex_xf = (start_x as f32 - left) * step_tx;
        let mut tex_yf_start = (start_y as f32 - top) * step_ty;

        let shade = (1.0 / (1.0 + perp * 0.001)).clamp(0.6, 1.0);

        let mut step_x_cols = 1;

        let mut x = start_x;
        while x <= end_x {
            if zbuf[x as usize] > perp {
                let mut tex_x = tex_xf.floor() as i32;
                if tex_x < 0 { tex_x = 0; }
                if tex_x >= fw { tex_x = fw - 1; }

                let mut y = start_y;
                let mut tex_yf = tex_yf_start;
                while y <= end_y {
                    let mut ty = tex_yf.floor() as i32;
                    if ty < 0 { ty = 0; }
                    if ty >= fh { ty = fh - 1; }

                    let px = x0 as i32 + tex_x;
                    let py = y0 as i32 + ty;
                    let idx = (((py as usize) * tw) + (px as usize)) * 4;

                    let r = tdata[idx];
                    let g = tdata[idx + 1];
                    let b = tdata[idx + 2];
                    let a = tdata[idx + 3];

                    if !(a == 0 || (r,g,b) == TRANSPARENT_KEY) {
                        let rr = (r as f32 * shade) as u8;
                        let gg = (g as f32 * shade) as u8;
                        let bb = (b as f32 * shade) as u8;
                        fb.put_pixel_rgba(x, y, rr, gg, bb, a);
                    }

                    y += 1;
                    tex_yf += step_ty;
                }

                if step_x_cols > 1 {
                    for rx in 1..step_x_cols {
                        let xx = x + rx as i32;
                        if xx > end_x { break; }
                        if zbuf[xx as usize] <= perp { break; }

                        let mut y2 = start_y;
                        let mut tex_yf2 = tex_yf_start;
                        while y2 <= end_y {
                            let mut ty = tex_yf2.floor() as i32;
                            if ty < 0 { ty = 0; }
                            if ty >= fh { ty = fh - 1; }

                            let px = x0 as i32 + tex_x;
                            let py = y0 as i32 + ty;
                            let idx = (((py as usize) * tw) + (px as usize)) * 4;

                            let r = tdata[idx];
                            let g = tdata[idx + 1];
                            let b = tdata[idx + 2];
                            let a = tdata[idx + 3];

                            if !(a == 0 || (r,g,b) == TRANSPARENT_KEY) {
                                let rr = (r as f32 * shade) as u8;
                                let gg = (g as f32 * shade) as u8;
                                let bb = (b as f32 * shade) as u8;
                                fb.put_pixel_rgba(xx, y2, rr, gg, bb, a);
                            }

                            y2 += 1;
                            tex_yf2 += step_ty;
                        }
                    }
                }
            }

            x += step_x_cols as i32;
            tex_xf += step_tx * step_x_cols as f32;
        }
    }
}

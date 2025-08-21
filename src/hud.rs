use rand::Rng;
use raylib::prelude::Vector2;

use crate::framebuffer::Framebuffer;
use crate::textures::TextureManager;
use crate::renderer::{render_minimap_zoomed, MinimapColors};
use crate::maze::Maze;
use crate::player::Player;
use crate::enemy::Enemy;

/// HUD de 128px, textura de fondo 'h', cara 'f' centrada y mini-mapa “zoom cam” con rayos.
pub struct Hud {
    pub height: i32,          // px
    pub face_playing: bool,
    pub face_time: f32,
    pub face_fps: f32,
    pub face_frames: usize,
    pub face_cooldown: f32,
    pub face_min_cd: f32,
    pub face_max_cd: f32,
    pub face_rect_w: i32,
    pub face_rect_h: i32,
    pub minimap_style: MinimapColors, // colores configurables

    /// ⬇️ NUEVO: tamaño de ventana del minimapa en celdas (ancho x alto)
    /// Esto controla el "zoom". Menos celdas => más zoom.
    pub minimap_cells_w: i32,
    pub minimap_cells_h: i32,
}

impl Hud {
    pub fn new(tex: &TextureManager) -> Self {
        let frames = tex.sheet_frames('f').max(1);
        Self {
            height: 128,
            face_playing: false,
            face_time: 0.0,
            face_fps: 4.0,
            face_frames: frames,
            face_cooldown: 1.5,
            face_min_cd: 3.0,
            face_max_cd: 7.0,
            face_rect_w: 128,
            face_rect_h: 128,
            minimap_style: MinimapColors::default(),
            // ventana inicial: ~11x9 celdas (ajústalo a gusto)
            minimap_cells_w: 11,
            minimap_cells_h: 9,
        }
    }

    pub fn update(&mut self, dt: f32) {
        if self.face_playing {
            self.face_time += dt;
            let total_anim = (self.face_frames as f32) / self.face_fps;
            if self.face_time >= total_anim {
                self.face_playing = false;
                self.face_time = 0.0;
                self.face_cooldown =
                    rand::thread_rng().gen_range(self.face_min_cd..=self.face_max_cd);
            }
        } else {
            self.face_cooldown -= dt;
            if self.face_cooldown <= 0.0 {
                self.face_playing = true;
                self.face_time = 0.0;
            }
        }
    }

    pub fn render(
        &self,
        fb: &mut Framebuffer,
        tex: &TextureManager,
        maze: &Maze,
        player: &Player,
        enemies: &[Enemy],
        block_size: usize,
    ) {
        let w = fb.width as i32;
        let h = fb.height as i32;
        let y0 = h - self.height;

        // 1) Fondo del HUD
        blit_image_to_rect(fb, tex, 'h', 0, y0, w, self.height);

        // 2) Cara en el centro
        if self.face_frames > 0 {
            let dst_w = self.face_rect_w.min(w);
            let dst_h = self.face_rect_h.min(self.height);
            let center_x = w / 2;
            let dst_x = center_x - dst_w / 2;
            let dst_y = y0 + (self.height - dst_h) / 2;

            let frame_idx = if self.face_playing {
                ((self.face_time * self.face_fps).floor() as usize) % self.face_frames
            } else {
                0
            };

            blit_sheet_frame_to_rect(fb, tex, 'f', frame_idx, dst_x, dst_y, dst_w, dst_h);
        }

        // 3) Mini-mapa con rayos (esquina inferior izquierda) — ahora tipo “ventana”
        let pad = 12;
        let mm_h = (self.height - pad * 2).max(1);
        let mm_w = (mm_h as f32 * 1.25) as i32; // un pelín ancho
        let mm_x = pad;
        let mm_y = y0 + pad;

        // Fondo suave del minimapa
        fill_rect(
            fb,
            mm_x - 2,
            mm_y - 2,
            mm_w + 4,
            mm_h + 4,
            self.minimap_style.frame,
        );

        render_minimap_zoomed(
            fb,
            maze,
            player,
            enemies,
            block_size,
            mm_x,
            mm_y,
            mm_w,
            mm_h,
            self.minimap_cells_w,
            self.minimap_cells_h,
            &self.minimap_style,
        );
    }
}

// Helpers de blit/fill

pub fn blit_image_to_rect(
    fb: &mut Framebuffer,
    tex: &TextureManager,
    key: char,
    dx: i32,
    dy: i32,
    dw: i32,
    dh: i32,
) {
    let (sw, sh, data) = tex.tex_view(key);
    if sw == 0 || sh == 0 || dw <= 0 || dh <= 0 {
        return;
    }

    for y in 0..dh {
        let sy = ((y as f32 / dh as f32) * sh as f32).floor() as i32;
        let sy = sy.clamp(0, sh as i32 - 1);
        for x in 0..dw {
            let sx = ((x as f32 / dw as f32) * sw as f32).floor() as i32;
            let sx = sx.clamp(0, sw as i32 - 1);
            let idx = (((sy as usize) * sw) + (sx as usize)) * 4;
            let r = data[idx];
            let g = data[idx + 1];
            let b = data[idx + 2];
            let a = data[idx + 3];
            fb.put_pixel_rgba(dx + x, dy + y, r, g, b, a);
        }
    }
}

pub fn blit_sheet_frame_to_rect(
    fb: &mut Framebuffer,
    tex: &TextureManager,
    key: char,
    frame: usize,
    dx: i32,
    dy: i32,
    dw: i32,
    dh: i32,
) {
    let (tw, th, x0, y0, fw, fh, data) = tex.sheet_frame_view(key, frame);
    if fw == 0 || fh == 0 || dw <= 0 || dh <= 0 {
        return;
    }

    for y in 0..dh {
        let sy = ((y as f32 / dh as f32) * fh as f32).floor() as i32;
        let sy = sy.clamp(0, fh as i32 - 1);
        for x in 0..dw {
            let sx = ((x as f32 / dw as f32) * fw as f32).floor() as i32;
            let sx = sx.clamp(0, fw as i32 - 1);
            let px = x0 as i32 + sx;
            let py = y0 as i32 + sy;
            let idx = (((py as usize) * tw) + (px as usize)) * 4;
            let r = data[idx];
            let g = data[idx + 1];
            let b = data[idx + 2];
            let a = data[idx + 3];
            fb.put_pixel_rgba(dx + x, dy + y, r, g, b, a);
        }
    }
}

fn fill_rect(fb: &mut Framebuffer, dx: i32, dy: i32, dw: i32, dh: i32, color: (u8, u8, u8, u8)) {
    let max_x = (dx + dw).min(fb.width);
    let max_y = (dy + dh).min(fb.height);
    for yy in dy.max(0)..max_y {
        for xx in dx.max(0)..max_x {
            fb.put_pixel_rgba(xx, yy, color.0, color.1, color.2, color.3);
        }
    }
}

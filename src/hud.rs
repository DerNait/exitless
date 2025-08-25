use rand::Rng;
use raylib::prelude::Vector2;

use crate::framebuffer::Framebuffer;
use crate::textures::TextureManager;
use crate::renderer::{render_minimap_zoomed, MinimapColors};
use crate::maze::Maze;
use crate::player::Player;
use crate::enemy::Enemy;
use crate::sprites::Sprite;
use crate::level::{LevelTheme, minimap_colors_for};

/// Dirección de los slots de llaves
#[derive(Clone, Copy)]
pub enum KeySlotsDirection { Row, Column }

/// Esquina/ancla reutilizable
#[derive(Clone, Copy)]
pub enum KeySlotsAnchor {
    BottomRight,
    BottomLeft,
    TopRight,
    TopLeft,
    /// Coordenadas absolutas (x,y) relativas a la pantalla
    Custom { x: i32, y: i32 },
}

/// Estilo visual del contenedor/slots (compat),
/// para las llaves dejamos todo transparente: sólo sprites.
#[derive(Clone, Copy)]
pub struct KeySlotsStyle {
    pub container_bg: (u8,u8,u8,u8), // ignorado (transparente)
    pub slot_bg:      (u8,u8,u8,u8), // ignorado (transparente)
    pub border:       (u8,u8,u8,u8), // ignorado
    pub border_px:    i32,           // ignorado
    pub icon_inset:   i32,           // margen interno del icono
}

/// Configuración del contenedor de llaves (posicionamiento y layout)
#[derive(Clone, Copy)]
pub struct KeySlotsConfig {
    pub slot: i32,            // tamaño base del icono (px)
    pub gap: i32,             // separación entre iconos
    pub pad: i32,             // margen respecto al borde HUD/pantalla
    pub container_pad: i32,   // ignorado (transparente)
    pub dir: KeySlotsDirection,
    pub anchor: KeySlotsAnchor,
    pub offset_x: i32,        // ajuste fino X
    pub offset_y: i32,        // ajuste fino Y
    pub style: KeySlotsStyle, // mantiene icon_inset
    /// Orden de los iconos (texturas HUD): 'y' (amarilla), 'b' (azul), 'r' (roja)
    pub order: [char; 3],
}

/// Configuración para los dígitos FPS (posición/tamaño/espaciado)
#[derive(Clone, Copy)]
pub struct FpsDigitsConfig {
    pub digit_w: i32,         // ancho de cada dígito en pantalla
    pub digit_h: i32,         // alto de cada dígito en pantalla
    pub gap: i32,             // separación entre dígitos
    pub max_digits: usize,    // reserva (típicamente 3: 0..999)
    pub anchor: KeySlotsAnchor, // esquina/ancla
    pub pad: i32,             // margen desde la esquina del HUD
    pub offset_x: i32,        // ajuste fino X
    pub offset_y: i32,        // ajuste fino Y
    /// Si true, el número se alinea a la derecha dentro del rect reservado
    /// (útil para que las unidades queden fijas aunque cambie el ancho)
    pub align_right: bool,
}

/// HUD de 128px, textura de fondo 'h', cara 'f', minimapa, llaves y FPS.
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

    /// Ventana del minimapa en celdas (ancho x alto) — controla zoom.
    pub minimap_cells_w: i32,
    pub minimap_cells_h: i32,

    /// Llaves (solo sprites con alpha, sin fondos)
    pub key_cfg: KeySlotsConfig,

    /// FPS digits config
    pub fps_cfg: FpsDigitsConfig,

    // --- Estado interno para FPS (suavizado/refresh) ---
    fps_accum_time: f32,
    fps_accum_frames: u32,
    fps_display: u32,
    fps_refresh: f32, // cada cuánto (seg) actualizar fps_display
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

            // Llaves: solo sprites transparentes (sin placa/fondo/borde)
            key_cfg: KeySlotsConfig {
                slot: 100,
                gap: 8,
                pad: 8,
                container_pad: 0,
                dir: KeySlotsDirection::Row,
                anchor: KeySlotsAnchor::BottomRight,
                offset_x: -20,
                offset_y: 0,
                style: KeySlotsStyle {
                    container_bg: (0, 0, 0, 0),
                    slot_bg:      (0, 0, 0, 0),
                    border:       (0, 0, 0, 0),
                    border_px:    0,
                    icon_inset:   0,
                },
                order: ['y', 'b', 'r'],
            },

            // FPS: 3 dígitos, arriba-derecha por defecto (tú lo ajustas)
            fps_cfg: FpsDigitsConfig {
                digit_w: 62,
                digit_h: 62,
                gap: 2,
                max_digits: 3,
                anchor: KeySlotsAnchor::TopRight,
                pad: 10,
                offset_x: -626, // ej: un pequeño ajuste para “colar” con tu HUD
                offset_y: 6,
                align_right: true,
            },

            // estado FPS
            fps_accum_time: 0.0,
            fps_accum_frames: 0,
            fps_display: 0,
            fps_refresh: 0.25, // refresco 4Hz (suavizado)
        }
    }

    /// Llama siempre cada frame (aunque no estés en Playing) para refrescar FPS.
    pub fn update(&mut self, dt: f32) {
        // Cara anim
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

        // FPS acumulado y refresco temporal
        self.fps_accum_time += dt;
        self.fps_accum_frames += 1;

        if self.fps_accum_time >= self.fps_refresh {
            // promedio por ventana → FPS “suavizado”
            let avg_dt = (self.fps_accum_time / (self.fps_accum_frames as f32)).max(1e-6);
            let fps = (1.0 / avg_dt).round();
            self.fps_display = fps.max(0.0) as u32;

            self.fps_accum_time = 0.0;
            self.fps_accum_frames = 0;
        }
    }

    pub fn render(
        &self,
        fb: &mut Framebuffer,
        tex: &TextureManager,
        maze: &Maze,
        player: &Player,
        enemies: &[Enemy],
        keys_sprites: &[Sprite], // para pintar llaves en minimapa
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

        // 3) Mini-mapa (abajo-izquierda)
        let pad = 6;
        let mm_h = (self.height - pad * 2).max(1);
        let mm_w = (mm_h as f32 * 1.25) as i32;
        let mm_x = pad;
        let mm_y = y0 + pad;

        fill_rect(fb, mm_x - 2, mm_y - 2, mm_w + 4, mm_h + 4, self.minimap_style.frame);
        
        render_minimap_zoomed(
            fb, maze, player, enemies, keys_sprites,
            block_size, mm_x, mm_y, mm_w, mm_h,
            self.minimap_cells_w, self.minimap_cells_h, &self.minimap_style,
        );

        // 4) Llaves (solo sprites con alpha + blending)
        self.render_key_icons_only(fb, tex, player, w, y0);

        // 5) FPS (dígitos desde spritesheet 'n' 0..9; sólo números)
        self.render_fps_digits(fb, tex, w, y0);
    }

    /// Llaves: ÚNICAMENTE los sprites de las llaves presentes (alpha blending).
    fn render_key_icons_only(
        &self,
        fb: &mut Framebuffer,
        tex: &TextureManager,
        player: &Player,
        screen_w: i32,
        hud_y0: i32,
    ) {
        let cfg = self.key_cfg;

        let slot = cfg.slot.max(8);
        let gap  = cfg.gap.max(0);
        let pad  = cfg.pad.max(0);
        let inset = cfg.style.icon_inset.clamp(0, slot / 3);

        let n = 3;
        let (content_w, content_h) = match cfg.dir {
            KeySlotsDirection::Row    => (slot * n + gap * (n - 1), slot),
            KeySlotsDirection::Column => (slot,         slot * n + gap * (n - 1)),
        };

        let (mut sx0, mut sy0) = match cfg.anchor {
            KeySlotsAnchor::BottomRight => (
                screen_w - pad - content_w,
                hud_y0 + self.height - pad - content_h
            ),
            KeySlotsAnchor::BottomLeft => (
                pad,
                hud_y0 + self.height - pad - content_h
            ),
            KeySlotsAnchor::TopRight => (
                screen_w - pad - content_w,
                hud_y0 + pad
            ),
            KeySlotsAnchor::TopLeft => (
                pad,
                hud_y0 + pad
            ),
            KeySlotsAnchor::Custom { x, y } => (x, y),
        };

        sx0 += cfg.offset_x;
        sy0 += cfg.offset_y;

        for (idx, key_ch) in cfg.order.iter().enumerate() {
            let (x, y) = match cfg.dir {
                KeySlotsDirection::Row    => (sx0 + idx as i32 * (slot + gap), sy0),
                KeySlotsDirection::Column => (sx0, sy0 + idx as i32 * (slot + gap)),
            };

            let has_key = match *key_ch {
                'y' => player.inv.key_yellow,
                'b' => player.inv.key_blue,
                'r' => player.inv.key_red,
                _   => false,
            };

            if has_key {
                blit_image_to_rect_over(
                    fb, tex, *key_ch,
                    x + inset, y + inset,
                    slot - inset * 2, slot - inset * 2
                );
            }
        }
    }

    /// FPS: dibuja los dígitos usando spritesheet 'n' (10 frames 0..9),
    /// con alpha blending y posición configurable.
    fn render_fps_digits(
        &self,
        fb: &mut Framebuffer,
        tex: &TextureManager,
        screen_w: i32,
        hud_y0: i32,
    ) {
        let cfg = self.fps_cfg;

        // Número mostrado y dígitos
        let mut fps = self.fps_display.min(999); // max 3 dígitos
        let mut digits: [u8; 3] = [0,0,0];
        let mut count = 0usize;

        // Al menos un dígito
        if fps == 0 {
            digits[0] = 0; count = 1;
        } else {
            while fps > 0 && count < digits.len() {
                digits[count] = (fps % 10) as u8;
                fps /= 10;
                count += 1;
            }
            // invertir (ahora están al revés)
            digits[..count].reverse();
        }

        // Medidas
        let dw = cfg.digit_w.max(1);
        let dh = cfg.digit_h.max(1);
        let gap = cfg.gap.max(0);
        let total_w = (count as i32) * dw + (count.saturating_sub(1) as i32) * gap;
        let total_h = dh;

        // Rectángulo reservado (según max_digits)
        let res_w = (cfg.max_digits as i32) * dw + ((cfg.max_digits.saturating_sub(1)) as i32) * gap;
        let res_h = dh;

        // Punto base por anchor (usamos el rectángulo reservado para que no "salte")
        let (mut base_x, mut base_y) = match cfg.anchor {
            KeySlotsAnchor::BottomRight => (
                screen_w - cfg.pad - res_w,
                hud_y0 + self.height - cfg.pad - res_h
            ),
            KeySlotsAnchor::BottomLeft => (
                cfg.pad,
                hud_y0 + self.height - cfg.pad - res_h
            ),
            KeySlotsAnchor::TopRight => (
                screen_w - cfg.pad - res_w,
                hud_y0 + cfg.pad
            ),
            KeySlotsAnchor::TopLeft => (
                cfg.pad,
                hud_y0 + cfg.pad
            ),
            KeySlotsAnchor::Custom { x, y } => (x, y),
        };

        base_x += cfg.offset_x;
        base_y += cfg.offset_y;

        // Alineación interna
        let start_x = if cfg.align_right {
            base_x + (res_w - total_w)
        } else {
            base_x
        };
        let start_y = base_y;

        // Pintar cada dígito (sheet 'n', frame = dígito 0..9) con blending
        for i in 0..count {
            let dx = start_x + i as i32 * (dw + gap);
            blit_sheet_frame_to_rect_over(fb, tex, 'n', digits[i] as usize, dx, start_y, dw, dh);
        }
    }

    pub fn apply_theme(&mut self, theme: &LevelTheme) {
        self.minimap_style = minimap_colors_for(theme);
    }
}

// -----------------------------------------------------------------------------
// Helpers de blit/fill (+ alpha blending)
// -----------------------------------------------------------------------------

/// Copia una imagen a rectángulo con alpha BLENDING (source-over) sobre el framebuffer.
pub fn blit_image_to_rect_over(
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

            let sr = data[idx];
            let sg = data[idx + 1];
            let sb = data[idx + 2];
            let sa = data[idx + 3];
            put_pixel_rgba_over(fb, dx + x, dy + y, sr, sg, sb, sa);
        }
    }
}

/// Copia un frame de spritesheet con alpha BLENDING (source-over).
pub fn blit_sheet_frame_to_rect_over(
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

            let sr = data[idx];
            let sg = data[idx + 1];
            let sb = data[idx + 2];
            let sa = data[idx + 3];
            put_pixel_rgba_over(fb, dx + x, dy + y, sr, sg, sb, sa);
        }
    }
}

/// Alpha blending (source-over) de un píxel RGBA sobre el framebuffer.
/// out_rgb = src_rgb * a + dst_rgb * (1-a)
/// out_a   = 255 (buffer final opaco)
#[inline]
pub fn put_pixel_rgba_over(
    fb: &mut Framebuffer,
    x: i32,
    y: i32,
    sr: u8,
    sg: u8,
    sb: u8,
    sa: u8,
) {
    if x < 0 || y < 0 || x >= fb.width || y >= fb.height { return; }

    let idx = ((y as usize * fb.width as usize) + x as usize) * 4;
    unsafe {
        let base = fb.color_buffer.data as *mut u8;

        let dr = *base.add(idx);
        let dg = *base.add(idx + 1);
        let db = *base.add(idx + 2);

        let a  = sa as u32;
        let ia = 255u32 - a;

        let rr = ((sr as u32 * a + dr as u32 * ia) / 255) as u8;
        let gg = ((sg as u32 * a + dg as u32 * ia) / 255) as u8;
        let bb = ((sb as u32 * a + db as u32 * ia) / 255) as u8;

        *base.add(idx)     = rr;
        *base.add(idx + 1) = gg;
        *base.add(idx + 2) = bb;
        *base.add(idx + 3) = 255;
    }
}

// (helpers sin blending, por compat)
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
    if sw == 0 || sh == 0 || dw <= 0 || dh <= 0 { return; }

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
    if fw == 0 || fh == 0 || dw <= 0 || dh <= 0 { return; }

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
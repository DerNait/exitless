// src/sprites.rs
use raylib::prelude::Vector2;

use crate::framebuffer::Framebuffer;
use crate::maze::Maze;
use crate::player::Player;
use crate::textures::TextureManager;

const PI: f32 = std::f32::consts::PI;
const TWO_PI: f32 = std::f32::consts::TAU;

// Color clave para transparencia (por si tu PNG no tiene alpha)
const TRANSPARENT_KEY: (u8, u8, u8) = (152, 0, 136);

#[derive(Clone, Copy, Debug)]
pub struct Sprite {
    pub pos: Vector2,  // coordenadas en píxeles del framebuffer (como el player)
    pub tex: char,     // id de textura en TextureManager (por ej. 'e')
    pub scale: f32,    // 1.0 = tamaño base (≈ block_size)
}

impl Sprite {
    pub fn new(pos: Vector2, tex: char, scale: f32) -> Self {
        Self { pos, tex, scale }
    }
}

/// Extrae sprites desde el Maze: cualquier celda con 'e' se convierte en sprite.
/// (La celda sigue siendo transitable; el raycaster NO debe tratarlos como pared.)
pub fn collect_sprites(maze: &Maze, block_size: usize) -> Vec<Sprite> {
    let mut v = Vec::new();
    for (j, row) in maze.iter().enumerate() {
        for (i, &c) in row.iter().enumerate() {
            if c == 'e' {
                let x = (i * block_size + block_size / 2) as f32;
                let y = (j * block_size + block_size / 2) as f32;
                v.push(Sprite::new(Vector2::new(x, y), 'e', 1.0));
            }
        }
    }
    v
}

#[inline]
fn normalize_angle(mut a: f32) -> f32 {
    while a >  PI { a -= TWO_PI; }
    while a < -PI { a += TWO_PI; }
    a
}

/// Renderiza todos los sprites con z‑buffer y orden por distancia.
pub fn render_sprites(
    fb: &mut Framebuffer,
    player: &Player,
    sprites: &[Sprite],
    tex: &TextureManager,
    zbuf: &[f32],           // z-buffer (una distancia por columna de pantalla)
    block_size: usize,
) {
    if sprites.is_empty() { return; }

    let w = fb.width as i32;
    let h = fb.height as i32;
    let hw = w as f32 * 0.5;
    let hh = h as f32 * 0.5;

    // Distancia al plano de proyección (misma que usas en world3d)
    let dist_to_plane = hw / (player.fov * 0.5).tan();

    // Ordenar back-to-front (lejos -> cerca)
    let mut order: Vec<(usize, f32)> = sprites.iter().enumerate()
        .map(|(idx, s)| {
            let dx = s.pos.x - player.pos.x;
            let dy = s.pos.y - player.pos.y;
            (idx, (dx*dx + dy*dy).sqrt())
        })
        .collect();
    order.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    for (idx, euclid_dist) in order {
        let s = sprites[idx];

        let dx = s.pos.x - player.pos.x;
        let dy = s.pos.y - player.pos.y;

        // 1) Ángulo hacia el sprite y diferencia con la vista del jugador
        let sprite_a = dy.atan2(dx);
        let mut da = normalize_angle(sprite_a - player.a);

        // Si está muy fuera del FOV, lo omitimos (pequeño margen para bordes)
        let limit = player.fov * 0.5 + 0.2;
        if da.abs() > limit { continue; }

        // 2) Distancia "perpendicular" (compensa fish-eye)
        //    Esto debe compararse contra el z-buffer y usarse para el tamaño.
        let perp_dist = (euclid_dist * da.cos()).max(1e-4);

        // 3) Tamaño proyectado (inversamente proporcional a la distancia)
        let base = block_size as f32 * s.scale;
        let sprite_h = ((base * dist_to_plane) / perp_dist).max(1.0);
        let sprite_w = sprite_h; // billboard cuadrado

        // 4) Posición horizontal en pantalla (centro + desplazamiento por tan(da))
        let screen_x = hw + da.tan() * dist_to_plane;

        // 5) Rectángulo en pantalla
        let start_x = (screen_x - sprite_w * 0.5).floor() as i32;
        let end_x   = (screen_x + sprite_w * 0.5).ceil()  as i32;

        let start_y = (hh - sprite_h * 0.5).floor() as i32;
        let end_y   = (hh + sprite_h * 0.5).ceil()  as i32;

        // Textura del sprite
        let (tw, th, tdata) = tex.tex_view(s.tex);
        let tw_i = tw as i32;
        let th_i = th as i32;

        // Sombrado sutil por distancia
        let shade = (1.0 / (1.0 + perp_dist * 0.001)).clamp(0.6, 1.0);

        // Barrido por columnas (x)
        let x0 = start_x.max(0);
        let x1 = end_x.min(w - 1);
        for x in x0..=x1 {
            // Oclusión con paredes: si la pared en esa columna está más cerca, salta
            if zbuf[x as usize] <= perp_dist { continue; }

            // Coord. de textura horizontal (0..tw-1)
            let tx = (((x - start_x) as f32) * tw as f32 / sprite_w) as i32;
            if tx < 0 || tx >= tw_i { continue; }

            // Barrido vertical (y)
            let y0 = start_y.max(0);
            let y1 = end_y.min(h - 1);
            for y in y0..=y1 {
                // Coord. de textura vertical (0..th-1)
                let ty = (((y - start_y) as f32) * th as f32 / sprite_h) as i32;
                if ty < 0 || ty >= th_i { continue; }

                let idx = ((ty as usize * tw) + tx as usize) * 4;
                let r = tdata[idx];
                let g = tdata[idx + 1];
                let b = tdata[idx + 2];
                let a = tdata[idx + 3];

                // Transparencia: alpha real o “magenta” clave
                let is_key = (r, g, b) == TRANSPARENT_KEY;
                if a == 0 || is_key { continue; }

                // Aplica sombreados simples por distancia
                let rr = (r as f32 * shade) as u8;
                let gg = (g as f32 * shade) as u8;
                let bb = (b as f32 * shade) as u8;

                fb.put_pixel_rgba(x, y, rr, gg, bb, a);
            }
        }
    }
}

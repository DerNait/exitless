use raylib::prelude::*; // Para RaylibDrawHandle en la función del título
use raylib::color::Color;
use crate::framebuffer::Framebuffer;
use crate::textures::TextureManager;
use crate::hud::{blit_image_to_rect, blit_image_to_rect_over};

/// Pantalla de inicio: logo 2px más pequeño (–2 en w/h) y centrado (+1,+1)
pub fn draw_start_screen(fb: &mut Framebuffer, tex: &TextureManager) {
    for y in 0..fb.height { fb.fill_row(y, Color::BLACK); }

    let (w,h) = (fb.width, fb.height);

    let mut lw = (w as f32 * 0.6) as i32;
    let mut lh = (lw as f32 * 0.35) as i32;

    let lx = (w - lw) / 2 + 1;
    let ly = (h - lh) / 3 + 1;

    blit_image_to_rect(fb, tex, 'O', lx, ly, lw, lh);
}

/// Selector de nivel: cards horizontales y assets 2px más pequeños (cards y candado).
pub fn draw_level_select(
    fb: &mut Framebuffer,
    tex: &TextureManager,
    selected: u8,
    unlocked: [bool;3],
) {
    // Fondo
    for y in 0..fb.height { fb.fill_row(y, Color::new(8,8,8,255)); }

    let (w,h) = (fb.width, fb.height);

    // --- Medidas para 3 cards en fila ---
    // Altura de card
    let mut card_h = (h as f32 * 0.45) as i32;
    // Separación horizontal
    let gap_x  = (w as f32 * 0.03) as i32;
    // Ancho de card calculado para 3 cards + 2 gaps
    let total_gap = gap_x * 2;
    let mut card_w = ((w - total_gap) as f32 / 3.0).floor() as i32;

    // Reducir 2px las imágenes y desfasar +1,+1 para centrar
    if card_w > 4 { card_w -= 4; }
    if card_h > 4 { card_h -= 4; }

    // Origen X centrado para [card, gap, card, gap, card]
    // OJO: como achicamos la card, el bloque es más chico: recalculemos
    let row_w = card_w * 3 + total_gap;
    let left_x = (w - row_w) / 2;

    // Centrado vertical (con el “-2px” ya aplicado)
    let y = (h - card_h) / 2 + 1; // +1 para compensar el shrink

    // Orden/índice → textura (ojo: no usar 'B' para evitar colisión con door_blue)
    let cards = [('A',0), ('N',1), ('C',2)];

    for (idx, (ch, i)) in cards.iter().enumerate() {
        let x = left_x + idx as i32 * (card_w + gap_x) + 1; // +1 para compensar el shrink

        // Marco si está seleccionada
        if *i as u8 == selected {
            let outline = Color::RAYWHITE;
            // top / bottom
            for xx in (x-8).max(0)..(x+card_w+8).min(w) {
                fb.put_pixel_rgba(xx, (y-8).max(0), outline.r,outline.g,outline.b,255);
                fb.put_pixel_rgba(xx, (y+card_h+8).min(h-1), outline.r,outline.g,outline.b,255);
            }
            // left / right
            for yy in (y-8).max(0)..(y+card_h+8).min(h) {
                fb.put_pixel_rgba((x-8).max(0), yy, outline.r,outline.g,outline.b,255);
                fb.put_pixel_rgba((x+card_w+8).min(w-1), yy, outline.r,outline.g,outline.b,255);
            }
        }

        // Card (–2px y centrada con +1,+1)
        blit_image_to_rect(fb, tex, *ch, x, y, card_w, card_h);

        // Candado si está bloqueada (también –2px total y centrado)
        if !unlocked[*i] {
            let mut lk_w = (card_w as f32 * 0.3) as i32;
            let mut lk_h = lk_w;
            if lk_w > 2 { lk_w -= 2; }
            if lk_h > 2 { lk_h -= 2; }

            let lk_x = x + (card_w - lk_w)/2 + 1;
            let lk_y = y + (card_h - lk_h)/2 + 1;
            blit_image_to_rect_over(fb, tex, 'K', lk_x, lk_y, lk_w, lk_h);
        }
    }
}

/// Dibuja el texto "SELECCIONA EL NIVEL" arriba, centrado.
/// Llama a esta función después de dibujar el framebuffer en `main.rs`,
/// cuando ya tengas `let mut d = rl.begin_drawing(&thread);`.
pub fn draw_level_select_header_text(d: &mut RaylibDrawHandle, screen_w: i32, font_size: i32) {
    let title = "SELECCIONA EL NIVEL";
    let tw = d.measure_text(title, font_size);
    let x = (screen_w - tw) / 2;
    let y = (font_size as f32 * 2.0) as i32; // margen superior agradable
    d.draw_text(title, x, y, font_size, Color::RAYWHITE);
}

use raylib::prelude::*;
use crate::player::Player;

pub fn process_events(window: &RaylibHandle, player: &mut Player) {
    const MOVE_SPEED: f32 = 90.0 / 60.0;     // píxeles por frame aprox.
    const ROT_SPEED:  f32 = std::f32::consts::PI / 120.0;

    // Rotación
    if window.is_key_down(KeyboardKey::KEY_LEFT)  || window.is_key_down(KeyboardKey::KEY_A) {
        player.a -= ROT_SPEED;
    }
    if window.is_key_down(KeyboardKey::KEY_RIGHT) || window.is_key_down(KeyboardKey::KEY_D) {
        player.a += ROT_SPEED;
    }

    // Movimiento adelante/atrás en dirección de vista
    let cos = player.a.cos();
    let sin = player.a.sin();
    if window.is_key_down(KeyboardKey::KEY_UP)   || window.is_key_down(KeyboardKey::KEY_W) {
        player.pos.x += MOVE_SPEED * cos;
        player.pos.y += MOVE_SPEED * sin;
    }
    if window.is_key_down(KeyboardKey::KEY_DOWN) || window.is_key_down(KeyboardKey::KEY_S) {
        player.pos.x -= MOVE_SPEED * cos;
        player.pos.y -= MOVE_SPEED * sin;
    }
}

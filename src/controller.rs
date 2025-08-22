use raylib::prelude::*;
use crate::player::Player;

// controller.rs
pub fn process_events(window: &RaylibHandle, player: &mut Player, dt: f32) {
    const MOVE_SPEED: f32 = 90.0; // px/seg
    const ROT_SPEED:  f32 = std::f32::consts::PI; // rad/seg (≈180°/s)

    if window.is_key_down(KeyboardKey::KEY_LEFT)  || window.is_key_down(KeyboardKey::KEY_A) {
        player.a -= ROT_SPEED * dt;
    }
    if window.is_key_down(KeyboardKey::KEY_RIGHT) || window.is_key_down(KeyboardKey::KEY_D) {
        player.a += ROT_SPEED * dt;
    }

    let cos = player.a.cos();
    let sin = player.a.sin();
    if window.is_key_down(KeyboardKey::KEY_UP)   || window.is_key_down(KeyboardKey::KEY_W) {
        player.pos.x += MOVE_SPEED * dt * cos;
        player.pos.y += MOVE_SPEED * dt * sin;
    }
    if window.is_key_down(KeyboardKey::KEY_DOWN) || window.is_key_down(KeyboardKey::KEY_S) {
        player.pos.x -= MOVE_SPEED * dt * cos;
        player.pos.y -= MOVE_SPEED * dt * sin;
    }
}


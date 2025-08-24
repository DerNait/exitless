use raylib::prelude::*;
use crate::player::Player;

#[inline]
fn wrap_angle(mut a: f32) -> f32 {
    use std::f32::consts::TAU;
    while a >  std::f32::consts::PI { a -= TAU; }
    while a < -std::f32::consts::PI { a += TAU; }
    a
}

pub fn process_events(
    window: &mut RaylibHandle,
    player: &mut Player,
    dt: f32,
    screen_w: i32,
    screen_h: i32,
) {
    const MOVE_SPEED: f32 = 80.0;    // px/s
    const SPRINT_MULT: f32 = 1.5;    // 50% más rápido
    const MOUSE_SENS: f32 = 0.0010;  // rad/pixel
    const DEADZONE:   f32 = 0.01;    // para evitar jitter

    // --- Mouse look por "warp-to-center" ---
    if window.is_window_focused() {
        let cx = (screen_w as f32) * 0.5;
        let cy = (screen_h as f32) * 0.5;

        let mp = window.get_mouse_position();
        let dx = mp.x - cx;

        if dx.abs() > DEADZONE {
            player.a = wrap_angle(player.a + dx * MOUSE_SENS);
        }

        window.set_mouse_position(Vector2::new(cx, cy));
    }

    // --- Movimiento WASD / Flechas ---
    let forward =
        (window.is_key_down(KeyboardKey::KEY_W) || window.is_key_down(KeyboardKey::KEY_UP)) as i32
      - (window.is_key_down(KeyboardKey::KEY_S) || window.is_key_down(KeyboardKey::KEY_DOWN)) as i32;

    let strafe =
        (window.is_key_down(KeyboardKey::KEY_D) || window.is_key_down(KeyboardKey::KEY_RIGHT)) as i32
      - (window.is_key_down(KeyboardKey::KEY_A) || window.is_key_down(KeyboardKey::KEY_LEFT)) as i32;

    if forward != 0 || strafe != 0 {
        let ca = player.a.cos();
        let sa = player.a.sin();
        let fx = ca;  let fy = sa;   // forward
        let rx = -sa; let ry = ca;   // right

        let mut mx = fx * (forward as f32) + rx * (strafe as f32);
        let mut my = fy * (forward as f32) + ry * (strafe as f32);

        let mag2 = mx*mx + my*my;
        if mag2 > 1e-6 {
            let inv = 1.0 / mag2.sqrt();
            mx *= inv; my *= inv;

            // sprint con SHIFT
            let mut speed = MOVE_SPEED;
            if window.is_key_down(KeyboardKey::KEY_LEFT_SHIFT) || window.is_key_down(KeyboardKey::KEY_RIGHT_SHIFT) {
                speed *= SPRINT_MULT;
            }

            player.pos.x += mx * speed * dt;
            player.pos.y += my * speed * dt;
        }
    }
}
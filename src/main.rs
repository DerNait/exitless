mod framebuffer;
mod maze;
mod renderer;
mod player;
mod caster;
mod controller;
mod world3d;
mod maze_gen; // opcional

use raylib::prelude::*;
use framebuffer::Framebuffer;
use maze::{load_maze, find_char, maze_dims, Maze};
use player::Player;
use renderer::render_maze;
use controller::process_events;
use caster::cast_ray;
use world3d::render_world;

fn main() {
    let screen_w = 900;
    let screen_h = 700;

    let (mut rl, thread) = raylib::init()
        .size(screen_w, screen_h)
        .title("Raycasting (2D/3D)")
        .build();

    let mut framebuffer = Framebuffer::new(screen_w, screen_h, Color::BLACK);

    // ---- Maze y block_size dinámico para llenar la ventana
    let maze: Maze = load_maze("assets/maze.txt");
    let (mw, mh) = maze_dims(&maze);
    let block_size_x = (screen_w as usize / mw).max(1);
    let block_size_y = (screen_h as usize / mh).max(1);
    let block_size = block_size_x.min(block_size_y); // cuadrado, ocupa lo máximo

    // ---- Player
    let (pi, pj) = find_char(&maze, 'p').unwrap_or((1, 1));
    let start_x = (pi * block_size + block_size / 2) as f32;
    let start_y = (pj * block_size + block_size / 2) as f32;
    let mut player = Player::new(
        Vector2::new(start_x, start_y),
        std::f32::consts::PI / 3.0,
        std::f32::consts::PI / 3.0,
    );

    // ---- Modo (2D/3D)
    let mut mode_3d = false;

    while !rl.window_should_close() {
        framebuffer.clear();

        // toggle de modo
        if rl.is_key_pressed(KeyboardKey::KEY_M) {
            mode_3d = !mode_3d;
        }

        // input
        process_events(&rl, &mut player);

        if mode_3d {
            // ----- RENDER 3D -----
            render_world(&mut framebuffer, &maze, &player, block_size);
        } else {
            // ----- RENDER 2D + rayos -----
            render_maze(&mut framebuffer, &maze, block_size);
            let num_rays = framebuffer.width; // 1 rayo por columna
            for i in 0..num_rays {
                let t = i as f32 / num_rays as f32;
                let a = player.a - (player.fov / 2.0) + (player.fov * t);
                // draw_line=true para ver el trazo en 2D
                let _ = cast_ray(&mut framebuffer, &maze, &player, block_size, a, true);
            }
        }

        // blit
        let tex = rl.load_texture_from_image(&thread, &framebuffer.color_buffer).unwrap();
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::BLACK);
        d.draw_texture(&tex, 0, 0, Color::WHITE);
        d.draw_text(
            if mode_3d { "M: 2D  | Flechas/WASD" } else { "M: 3D  | Flechas/WASD" },
            10, 10, 18, Color::RAYWHITE
        );
    }
}

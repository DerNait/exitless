// main.rs
mod framebuffer;
mod maze;
mod renderer;
mod player;
mod caster;
mod controller;
mod world3d;
mod textures;
mod maze_gen;

use raylib::prelude::*;
use raylib::consts::TextureFilter;
use raylib::core::texture::RaylibTexture2D; // <-- importa el trait para .update_texture()

use framebuffer::Framebuffer;
use maze::{load_maze, find_char, maze_dims, Maze};
use player::Player;
use renderer::render_maze;
use controller::process_events;
use world3d::render_world_textured;
use textures::TextureManager;

fn main() {
    let screen_w = 900;
    let screen_h = 700;

    let (mut rl, thread) = raylib::init()
        .size(screen_w, screen_h)
        .title("Raycasting (2D/3D + Texturas)")
        .build();

    let mut framebuffer = Framebuffer::new(screen_w, screen_h, Color::BLACK);

    // --- Texturas de paredes
    let tex_manager = TextureManager::new(&mut rl, &thread);

    // --- Maze y block_size
    let maze: Maze = load_maze("assets/maze.txt");
    let (mw, mh) = maze_dims(&maze);
    let block_size_x = (screen_w as usize / mw).max(1);
    let block_size_y = (screen_h as usize / mh).max(1);
    let block_size = block_size_x.min(block_size_y);

    // --- Player
    let (pi, pj) = find_char(&maze, 'p').unwrap_or((1, 1));
    let start_x = (pi * block_size + block_size / 2) as f32;
    let start_y = (pj * block_size + block_size / 2) as f32;
    let mut player = Player::new(
        Vector2::new(start_x, start_y),
        std::f32::consts::PI / 3.0,
        std::f32::consts::PI / 3.0,
    );

    // --- Modo 2D/3D
    let mut mode_3d = true;

    // --- Creamos una textura de pantalla UNA vez y la actualizamos cada frame
    let mut screen_tex = rl
        .load_texture_from_image(&thread, &framebuffer.color_buffer)
        .expect("No se pudo crear la textura de pantalla");
    screen_tex.set_texture_filter(&thread, TextureFilter::TEXTURE_FILTER_POINT);

    while !rl.window_should_close() {
        framebuffer.clear();

        if rl.is_key_pressed(KeyboardKey::KEY_M) {
            mode_3d = !mode_3d;
        }

        process_events(&rl, &mut player);

        if mode_3d {
            render_world_textured(&mut framebuffer, &maze, &player, block_size, &tex_manager);
        } else {
            render_maze(&mut framebuffer, &maze, block_size);
            let num_rays = framebuffer.width;
            for i in 0..num_rays {
                let t = i as f32 / num_rays as f32;
                let a = player.a - (player.fov / 2.0) + (player.fov * t);
                let _ = caster::cast_ray(&mut framebuffer, &maze, &player, block_size, a, true);
            }
        }

        // --- Actualizamos los píxeles de la textura ya existente
        unsafe {
            let len = (framebuffer.width * framebuffer.height * 4) as usize; // RGBA8
            let slice = std::slice::from_raw_parts(
                framebuffer.color_buffer.data as *const u8,
                len,
            );
            screen_tex.update_texture(slice).expect("update_texture falló");
        }

        // --- Dibujar
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::BLACK);
        d.draw_texture(&screen_tex, 0, 0, Color::WHITE);
        d.draw_text(
            if mode_3d { "M: 2D | Texturas ON | WASD/Flechas" } else { "M: 3D | WASD/Flechas" },
            10, 10, 18, Color::RAYWHITE,
        );
    }
}

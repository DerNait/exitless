mod framebuffer;
mod maze;
mod renderer;
mod player;
mod caster;
mod controller;
mod world3d;
mod textures;
mod maze_gen;
mod sprites;
mod enemy;        // 👈 nuevo
mod utils_grid;   // 👈 nuevo

use raylib::prelude::*;
use raylib::consts::TextureFilter;

use framebuffer::Framebuffer;
use maze::{load_maze, find_char, maze_dims, Maze};
use player::Player;
use renderer::render_maze;
use controller::process_events;
use world3d::render_world_textured;
use textures::TextureManager;
use sprites::{collect_sprites, Sprite};
use enemy::{Enemy, update_enemy};

fn main() {
    let screen_w = 1000;
    let screen_h = 800;
    let (mut rl, thread) = raylib::init()
        .size(screen_w, screen_h)
        .title("Raycasting con enemigos")
        .build();

    let mut framebuffer = Framebuffer::new(screen_w, screen_h, Color::BLACK);
    let tex_manager = TextureManager::new(&mut rl, &thread);

    let mut maze: Maze = load_maze("assets/maze.txt"); // mutable

    let (mw, mh) = maze_dims(&maze);
    let block_size_x = (screen_w as usize / mw).max(1);
    let block_size_y = (screen_h as usize / mh).max(1);
    let block_size = block_size_x.min(block_size_y);

    let (pi, pj) = find_char(&maze, 'p').unwrap_or((1, 1));
    let start_x = (pi * block_size + block_size / 2) as f32;
    let start_y = (pj * block_size + block_size / 2) as f32;
    let mut player = Player::new(
        Vector2::new(start_x, start_y),
        std::f32::consts::PI / 3.0,
        std::f32::consts::PI / 3.0,
    );

    // --- Enemigos ---
    let mut enemies: Vec<Enemy> = Vec::new();
    for (j,row) in maze.iter_mut().enumerate() {
        for (i,c) in row.iter_mut().enumerate() {
            if *c == 'e' {
                enemies.push(Enemy::from_cell(i as i32, j as i32, block_size));
                *c = ' '; // vuelve caminable
            }
        }
    }

    let sprites: Vec<Sprite> = collect_sprites(&maze, block_size, &tex_manager);

    let mut mode_3d = true;

    let mut screen_tex = rl
        .load_texture_from_image(&thread, &framebuffer.color_buffer)
        .expect("No se pudo crear la textura de pantalla");
    screen_tex.set_texture_filter(&thread, TextureFilter::TEXTURE_FILTER_POINT);

    let mut time_s: f32 = 0.0;

    while !rl.window_should_close() {
        let dt = rl.get_frame_time();
        time_s += dt;

        if rl.is_key_pressed(KeyboardKey::KEY_M) { mode_3d = !mode_3d; }
        process_events(&rl, &mut player);

        for e in &mut enemies {
            update_enemy(e, &maze, player.pos, block_size, dt);
        }

        if mode_3d {
            render_world_textured(
                &mut framebuffer,
                &maze,
                &player,
                block_size,
                &tex_manager,
                &sprites,
                &enemies,   // 👈 ahora también pasamos enemigos
                time_s,
            );
        } else {
            framebuffer.clear();
            render_maze(&mut framebuffer, &maze, block_size);
        }

        unsafe {
            let len = (framebuffer.width * framebuffer.height * 4) as usize;
            let slice = std::slice::from_raw_parts(
                framebuffer.color_buffer.data as *const u8,
                len,
            );
            screen_tex.update_texture(slice).unwrap();
        }

        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::BLACK);
        d.draw_texture(&screen_tex, 0, 0, Color::WHITE);
        d.draw_text(
            if mode_3d { "M: 2D | Texturas+Sprites+Enemigos ON" } else { "M: 3D | WASD/Flechas" },
            10, 10, 18, Color::RAYWHITE,
        );
    }
}

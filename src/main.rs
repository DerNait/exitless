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
mod enemy;
mod utils_grid;
mod gamemanager;
mod hud;

use raylib::prelude::*;
use raylib::consts::TextureFilter;

use framebuffer::Framebuffer;
use maze::{load_maze, find_char, maze_dims, Maze};
use player::Player;
use world3d::{render_world_textured, draw_overlay_fullscreen, draw_game_over_background};
use textures::TextureManager;
use sprites::{collect_sprites, Sprite};
use enemy::{Enemy, update_enemy};
use gamemanager::{GameManager, GameState};
use hud::Hud;

fn recreate_enemies(cells: &[(i32,i32)], block_size: usize) -> Vec<Enemy> {
    let mut v = Vec::with_capacity(cells.len());
    for &(ci, cj) in cells { v.push(Enemy::from_cell(ci, cj, block_size)); }
    v
}

fn main() {
    let screen_w = 1000;
    let screen_h = 800;
    let (mut rl, thread) = raylib::init()
        .size(screen_w, screen_h)
        .title("Raycasting con HUD + Minimap")
        .build();

    let mut framebuffer = Framebuffer::new(screen_w, screen_h, Color::BLACK);
    let tex_manager = TextureManager::new(&mut rl, &thread);

    let mut maze: Maze = load_maze("assets/maze.txt");

    let (mw, mh) = maze_dims(&maze);
    let block_size_x = (screen_w as usize / mw).max(1);
    let block_size_y = (screen_h as usize / mh).max(1);
    let block_size = block_size_x.min(block_size_y);

    // Spawns
    let (pi, pj) = find_char(&maze, 'p').unwrap_or((1, 1));
    let player_spawn_px = (
        (pi * block_size + block_size / 2) as f32,
        (pj * block_size + block_size / 2) as f32
    );
    let player_spawn_angle = std::f32::consts::PI / 3.0;
    let player_spawn_fov   = std::f32::consts::PI / 3.0;

    let mut enemy_spawn_cells: Vec<(i32,i32)> = Vec::new();
    for (j,row) in maze.iter().enumerate() {
        for (i,&c) in row.iter().enumerate() {
            if c == 'e' { enemy_spawn_cells.push((i as i32, j as i32)); }
        }
    }
    for row in maze.iter_mut() {
        for c in row.iter_mut() {
            if *c == 'e' { *c = ' '; }
        }
    }

    let mut player = Player::new(
        Vector2::new(player_spawn_px.0, player_spawn_px.1),
        player_spawn_angle,
        player_spawn_fov,
    );
    let mut enemies: Vec<Enemy> = recreate_enemies(&enemy_spawn_cells, block_size);
    let sprites: Vec<Sprite> = collect_sprites(&maze, block_size, &tex_manager);

    let mut screen_tex = rl
        .load_texture_from_image(&thread, &framebuffer.color_buffer)
        .expect("No se pudo crear la textura de pantalla");
    screen_tex.set_texture_filter(&thread, TextureFilter::TEXTURE_FILTER_POINT);

    let mut time_s: f32 = 0.0;

    // Game Manager
    let trigger_dist = (block_size as f32) * 0.85;
    let mut gm = GameManager::new(trigger_dist, 2.0);

    // HUD
    let mut hud = Hud::new(&tex_manager);

    // reset helper
    let mut do_reset = |player: &mut Player,
                        enemies: &mut Vec<Enemy>,
                        gm: &mut GameManager,
                        time_s: &mut f32,
                        hud: &mut Hud| {
        player.pos.x = player_spawn_px.0;
        player.pos.y = player_spawn_px.1;
        player.a     = player_spawn_angle;
        player.fov   = player_spawn_fov;
        *enemies = recreate_enemies(&enemy_spawn_cells, block_size);
        *time_s = 0.0;
        gm.reset();
        hud.face_playing = false;
        hud.face_time = 0.0;
        hud.face_cooldown = 1.5;
    };

    while !rl.window_should_close() {
        let dt = rl.get_frame_time();
        time_s += dt;

        if gm.is_game_over() && rl.is_key_pressed(KeyboardKey::KEY_R) {
            do_reset(&mut player, &mut enemies, &mut gm, &mut time_s, &mut hud);
        }

        if gm.is_playing() {
            crate::controller::process_events(&rl, &mut player);
            for e in &mut enemies { update_enemy(e, &maze, player.pos, block_size, dt); }
        }

        let enemy_positions = enemies.iter().map(|e| e.pos);
        gm.update(player.pos, enemy_positions, dt);

        if gm.is_playing() { hud.update(dt); }

        // Viewport del mundo 3D (excluye el HUD de 150px)
        let vp_y0 = 0;
        let vp_h  = framebuffer.height - hud.height;

        match gm.state {
            GameState::Playing => {
                render_world_textured(
                    &mut framebuffer,
                    &maze,
                    &player,
                    block_size,
                    &tex_manager,
                    &sprites,
                    &enemies,
                    time_s,
                    vp_y0,
                    vp_h,
                );
                hud.render(&mut framebuffer, &tex_manager, &maze, &player, &enemies, block_size);
            }
            GameState::JumpScare => {
                render_world_textured(
                    &mut framebuffer,&maze,&player,block_size,&tex_manager,&sprites,&[],time_s,
                    vp_y0, vp_h,
                );
                let fb_w = framebuffer.width;
                // Overlay SOLO en el viewport 3D → no cubre el HUD
                world3d::draw_overlay_viewport(
                    &mut framebuffer, &tex_manager, 'j',
                    0, vp_y0, fb_w, vp_h
                );
                hud.render(&mut framebuffer, &tex_manager, &maze, &player, &enemies, block_size);
            }

            GameState::GameOver => {
                draw_game_over_background(&mut framebuffer);
            }
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

        match gm.state {
            GameState::Playing => {
                d.draw_text("R: Reset", 10, 10, 18, Color::RAYWHITE);
            }
            GameState::JumpScare => { /* overlay tapa */ }
            GameState::GameOver => {
                let text = "GAME OVER";
                let font_size = 100;
                let tw = d.measure_text(text, font_size);
                let x = (screen_w - tw) / 2;
                let y = (screen_h - font_size) / 2;
                d.draw_text(text, x, y, font_size, Color::RED);

                let sub = "Presiona R para reiniciar - ESC para salir";
                let sub_size = 24;
                let sw = d.measure_text(sub, sub_size);
                let sx = (screen_w - sw) / 2;
                d.draw_text(sub, sx, y + font_size + 20, sub_size, Color::RAYWHITE);
            }
        }
    }
}

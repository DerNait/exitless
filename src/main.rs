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
mod gamemanager; // 👈 nuevo

use raylib::prelude::*;
use raylib::consts::TextureFilter;

use framebuffer::Framebuffer;
use maze::{load_maze, find_char, maze_dims, Maze};
use player::Player;
use renderer::render_maze;
use controller::process_events;
use world3d::{render_world_textured, draw_overlay_fullscreen, draw_game_over_background};
use textures::TextureManager;
use sprites::{collect_sprites, Sprite};
use enemy::{Enemy, update_enemy};
use gamemanager::{GameManager, GameState};

fn recreate_enemies(cells: &[(i32,i32)], block_size: usize) -> Vec<Enemy> {
    let mut v = Vec::with_capacity(cells.len());
    for &(ci, cj) in cells {
        v.push(Enemy::from_cell(ci, cj, block_size));
    }
    v
}

fn main() {
    let screen_w = 1000;
    let screen_h = 800;
    let (mut rl, thread) = raylib::init().size(screen_w, screen_h).title("Raycasting con enemigos").build();

    let mut framebuffer = Framebuffer::new(screen_w, screen_h, Color::BLACK);
    let tex_manager = TextureManager::new(&mut rl, &thread);

    // --- Maze base (inmutable en disco)
    let mut maze: Maze = load_maze("assets/maze.txt");

    let (mw, mh) = maze_dims(&maze);
    let block_size_x = (screen_w as usize / mw).max(1);
    let block_size_y = (screen_h as usize / mh).max(1);
    let block_size = block_size_x.min(block_size_y);

    // Spawns INICIALES
    let (pi, pj) = find_char(&maze, 'p').unwrap_or((1, 1));
    let player_spawn_px = (
        (pi * block_size + block_size / 2) as f32,
        (pj * block_size + block_size / 2) as f32
    );
    let player_spawn_angle = std::f32::consts::PI / 3.0;
    let player_spawn_fov   = std::f32::consts::PI / 3.0;

    // Celdas de enemigos iniciales (antes de limpiar el maze)
    let mut enemy_spawn_cells: Vec<(i32,i32)> = Vec::new();
    for (j,row) in maze.iter().enumerate() {
        for (i,&c) in row.iter().enumerate() {
            if c == 'e' {
                enemy_spawn_cells.push((i as i32, j as i32));
            }
        }
    }
    // Limpiamos 'e' para que el grid sea caminable
    for (j,row) in maze.iter_mut().enumerate() {
        for (i,c) in row.iter_mut().enumerate() {
            if *c == 'e' { *c = ' '; }
        }
    }

    // Estado vivo
    let mut player = Player::new(
        Vector2::new(player_spawn_px.0, player_spawn_px.1),
        player_spawn_angle,
        player_spawn_fov,
    );
    let mut enemies: Vec<Enemy> = recreate_enemies(&enemy_spawn_cells, block_size);

    // (Opcional: sprites decorativos si los usas)
    let sprites: Vec<Sprite> = collect_sprites(&maze, block_size, &tex_manager);

    let mut mode_3d = true;

    let mut screen_tex = rl.load_texture_from_image(&thread, &framebuffer.color_buffer)
        .expect("No se pudo crear la textura de pantalla");
    screen_tex.set_texture_filter(&thread, TextureFilter::TEXTURE_FILTER_POINT);

    let mut time_s: f32 = 0.0;

    // Game Manager
    let trigger_dist = (block_size as f32) * 0.65;
    let mut gm = GameManager::new(trigger_dist, 2.0);

    // --- función inline para RESET total ---
    let mut do_reset = |player: &mut Player,
                        enemies: &mut Vec<Enemy>,
                        gm: &mut GameManager,
                        time_s: &mut f32| {
        // Player
        player.pos.x = player_spawn_px.0;
        player.pos.y = player_spawn_px.1;
        player.a     = player_spawn_angle;
        player.fov   = player_spawn_fov;

        // Enemigos
        *enemies = recreate_enemies(&enemy_spawn_cells, block_size);

        // Tiempo y estados
        *time_s = 0.0;
        gm.reset();
    };

    while !rl.window_should_close() {
        let dt = rl.get_frame_time();
        time_s += dt;

        // Toggle modo solo si jugando
        if gm.is_playing() && rl.is_key_pressed(KeyboardKey::KEY_M) { mode_3d = !mode_3d; }

        // RESET si estás en GameOver y presionas R (o si prefieres permitirlo siempre)
        if gm.is_game_over() && rl.is_key_pressed(KeyboardKey::KEY_R) {
            do_reset(&mut player, &mut enemies, &mut gm, &mut time_s);
        }

        // Input + lógica de enemigo solo en Playing
        if gm.is_playing() {
            process_events(&rl, &mut player);
            for e in &mut enemies { update_enemy(e, &maze, player.pos, block_size, dt); }
        }

        // Avance de estado global
        let enemy_positions = enemies.iter().map(|e| e.pos);
        gm.update(player.pos, enemy_positions, dt);

        // --- Render ---
        if mode_3d {
            match gm.state {
                GameState::Playing => {
                    render_world_textured(&mut framebuffer,&maze,&player,block_size,&tex_manager,&sprites,&enemies,time_s);
                }
                GameState::JumpScare => {
                    render_world_textured(&mut framebuffer,&maze,&player,block_size,&tex_manager,&sprites,&[],time_s);
                    draw_overlay_fullscreen(&mut framebuffer, &tex_manager, 'j');
                }
                GameState::GameOver => {
                    draw_game_over_background(&mut framebuffer);
                }
            }
        } else {
            framebuffer.clear();
            render_maze(&mut framebuffer, &maze, block_size);
        }

        unsafe {
            let len = (framebuffer.width * framebuffer.height * 4) as usize;
            let slice = std::slice::from_raw_parts(framebuffer.color_buffer.data as *const u8, len);
            screen_tex.update_texture(slice).unwrap();
        }

        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::BLACK);
        d.draw_texture(&screen_tex, 0, 0, Color::WHITE);

        match gm.state {
            GameState::Playing => {
                d.draw_text("R: Reset", 10, 34, 18, Color::RAYWHITE);
                d.draw_text(if mode_3d {"M: 2D | Texturas+Sprites+Enemigos ON"} else {"M: 3D | WASD/Flechas"}, 10, 10, 18, Color::RAYWHITE);
            }
            GameState::JumpScare => { /* Overlay tapa todo */ }
            GameState::GameOver => {
                let text = "GAME OVER";
                let font_size = 100;
                let tw = d.measure_text(text, font_size);
                let x = (screen_w - tw) / 2;
                let y = (screen_h - font_size) / 2;
                d.draw_text(text, x, y, font_size, Color::RED);

                let sub = "Presiona R para reiniciar • ESC para salir";
                let sub_size = 24;
                let sw = d.measure_text(sub, sub_size);
                let sx = (screen_w - sw) / 2;
                d.draw_text(sub, sx, y + font_size + 20, sub_size, Color::RAYWHITE);
            }
        }
    }
}
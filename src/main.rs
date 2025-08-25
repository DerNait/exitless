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
mod physics;
mod audiomanager;

// NUEVOS
mod appstate;
mod save;
mod level;
mod menu;

use raylib::prelude::*;
use raylib::consts::TextureFilter;
use raylib::core::audio::RaylibAudio;

use audiomanager::{AudioManager, AudioConfig};

use framebuffer::Framebuffer;
use maze::{load_maze, find_char, maze_dims, Maze};
use player::Player;
use world3d::{render_world_textured, draw_overlay_fullscreen, draw_game_over_background, draw_win_background};
use textures::TextureManager;
use sprites::{collect_sprites, Sprite};
use enemy::{Enemy, update_enemy};
use gamemanager::{GameManager, GameState};
use hud::Hud;
use physics::resolve_player_collisions;

use appstate::AppState;
use save::{Progress, load_progress, save_progress};
use level::{LevelTheme, theme_for};
use menu::{draw_start_screen, draw_level_select};
use std::fs;

fn recreate_enemies(cells: &[(i32,i32)], block_size: usize) -> Vec<Enemy> {
    let mut v = Vec::with_capacity(cells.len());
    for &(ci, cj) in cells { v.push(Enemy::from_cell(ci, cj, block_size)); }
    v
}

fn forward_cell(pos: Vector2, ang: f32, block_size: usize, steps_px: f32) -> (i32,i32) {
    let nx = pos.x + ang.cos() * steps_px;
    let ny = pos.y + ang.sin() * steps_px;
    let ci = (nx / block_size as f32).floor() as i32;
    let cj = (ny / block_size as f32).floor() as i32;
    (ci, cj)
}

fn level_path(level: u8) -> &'static str {
    match level {
        0 => "assets/level1/maze.txt",
        1 => "assets/level2/maze.txt",
        _ => "assets/level3/maze.txt",
    }
}

/// Tamaños sugeridos por nivel (ajústalos libremente)
fn level_dims(level: u8) -> (usize, usize) {
    match level {
        0 => (10, 10),   // compacto
        1 => (15, 15),   // mediano
        _ => (20, 20),   // grande
    }
}

/// Genera y persiste el maze del nivel y luego lo carga.
fn load_maze_for_level(level: u8) -> Maze {
    use crate::maze_gen::{make_maze_text_advanced, MazeGenConfig};

    let path = level_path(level);
    let (w, h) = level_dims(level);

    // Puedes tunear por nivel si quieres variar
    let cfg = match level {
        0 => MazeGenConfig { loop_factor: 0.12, donuts: 2, special_border_prob: 0.04, keys_per_type_base: 10, doors_per_type_base: 5, seed: None },
        1 => MazeGenConfig { loop_factor: 0.16, donuts: 3, special_border_prob: 0.04, keys_per_type_base: 14, doors_per_type_base: 7, seed: None },
        _ => MazeGenConfig { loop_factor: 0.22, donuts: 4, special_border_prob: 0.04, keys_per_type_base: 20, doors_per_type_base: 10, seed: None },
    };

    let txt = make_maze_text_advanced(w, h, cfg);
    let _ = fs::create_dir_all(std::path::Path::new(path).parent().unwrap());
    let _ = fs::write(path, txt);

    // y lo cargamos con tu loader existente
    load_maze(path)
}

// -------- RESET SIN CAPTURAS (evita E0506/E0502) ----------
fn do_reset(
    player: &mut Player,
    enemies: &mut Vec<Enemy>,
    gm: &mut GameManager,
    time_s: &mut f32,
    hud: &mut Hud,
    maze_ref: &mut Maze,
    keys_ref: &mut Vec<Sprite>,
    enemy_cells_ref: &mut Vec<(i32,i32)>,
    audio_ref: &mut AudioManager,
    tex_manager: &TextureManager,
    maze_original: &Maze,
    block_size: usize,
    player_spawn_px: (f32, f32),
    player_spawn_angle: f32,
    player_spawn_fov: f32,
) {
    *maze_ref = maze_original.clone();

    enemy_cells_ref.clear();
    for (j,row) in maze_ref.iter().enumerate() {
        for (i,&c) in row.iter().enumerate() {
            if c == 'e' { enemy_cells_ref.push((i as i32, j as i32)); }
        }
    }
    for row in maze_ref.iter_mut() {
        for c in row.iter_mut() {
            if *c == 'e' { *c = ' '; }
        }
    }

    *keys_ref = sprites::collect_keys(maze_ref, block_size, tex_manager);
    for row in maze_ref.iter_mut() {
        for c in row.iter_mut() {
            if *c == '1' || *c == '2' || *c == '3' { *c = ' '; }
        }
    }

    player.pos.x = player_spawn_px.0;
    player.pos.y = player_spawn_px.1;
    player.a     = player_spawn_angle;
    player.fov   = player_spawn_fov;
    player.inv.key_yellow = false;
    player.inv.key_blue   = false;
    player.inv.key_red    = false;

    *enemies = recreate_enemies(enemy_cells_ref, block_size);

    *time_s = 0.0;
    gm.reset();
    hud.face_playing = false;
    hud.face_time = 0.0;
    hud.face_cooldown = 1.5;

    audio_ref.reset_to_game();
    audio_ref.on_state_changed(GameState::Playing);
}

fn main() {
    let screen_w = 1000;
    let screen_h = 800;
    let (mut rl, thread) = raylib::init()
        .size(screen_w, screen_h)
        .title("Raycasting - Menu y Niveles")
        .build();

    rl.set_target_fps(120);
    rl.hide_cursor();

    // Audio
    let ra = RaylibAudio::init_audio_device().expect("No se pudo inicializar audio");
    let audio_cfg = AudioConfig {
        master_music: 1.0,
        master_sfx:   1.0,

        vol_music_game:      1.0,
        vol_music_jumpscare: 1.0,
        vol_music_gameover:  0.9,
        vol_music_win:       0.9,

        vol_enemy_loop: 1.5,

        vol_sfx_door_open: 1.5,
        vol_sfx_key_pick:  1.0,
        vol_sfx_jumpscare: 1.0,

        fade_speed: 2.0,
        enemy_max_dist: 600.0,

        enemy_loop_in_playing:   true,
        enemy_loop_in_jumpscare: false,
        enemy_loop_in_gameover:  false,
        enemy_loop_in_win:       false,
    };
    let mut audio = AudioManager::new(&ra, audio_cfg);

    let mut framebuffer = Framebuffer::new(screen_w, screen_h, Color::BLACK);
    let mut tex_manager = TextureManager::new(&mut rl, &thread);

    // Progreso y estado de app
    let mut progress: Progress = load_progress();
    let mut app_state: AppState = AppState::StartScreen;

    // Tema actual (arrancamos con L1 para tener UI lista)
    let mut current_theme: LevelTheme = theme_for(0);
    tex_manager.apply_theme(&mut rl, &thread, &current_theme);
    audio.load_theme_music(&current_theme);

    // Mapa inicial = nivel 1 (se reemplaza al elegir)
    let mut maze_original: Maze = load_maze_for_level(0);
    let mut maze: Maze = maze_original.clone();

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

    // Enemigos desde mapa actual
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

    // Llaves desde mapa y limpiar
    let mut keys_sprites = sprites::collect_keys(&maze, block_size, &tex_manager);
    for row in maze.iter_mut() {
        for c in row.iter_mut() {
            if *c == '1' || *c == '2' || *c == '3' { *c = ' '; }
        }
    }

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
    hud.apply_theme(&current_theme);

    // Track de estado previo
    let mut prev_state = gm.state;

    while !rl.window_should_close() {
        let dt = rl.get_frame_time();
        time_s += dt;

        match app_state {
            AppState::StartScreen => {
                // --- INPUTS (antes de dibujar) ---
                let enter = rl.is_key_pressed(KeyboardKey::KEY_ENTER);

                // --- RENDER ---
                draw_start_screen(&mut framebuffer, &tex_manager);
                unsafe {
                    let len = (framebuffer.width * framebuffer.height * 4) as usize;
                    let slice = std::slice::from_raw_parts(
                        framebuffer.color_buffer.data as *const u8, len);
                    screen_tex.update_texture(slice).unwrap();
                }
                {
                    let mut d = rl.begin_drawing(&thread);
                    d.clear_background(Color::BLACK);
                    d.draw_texture(&screen_tex, 0, 0, Color::WHITE);

                    let msg = "Presiona ENTER para comenzar";
                    let fs = 28;
                    let tw = d.measure_text(msg, fs);
                    d.draw_text(msg, (screen_w - tw)/2, (screen_h*2)/3 + 160, fs, Color::RAYWHITE);
                } // <- Drop de d aquí

                // --- TRANSICIONES ---
                if enter {
                    app_state = AppState::LevelSelect { selected: progress.last_level.min(2) };
                    rl.show_cursor();
                }
            }

            AppState::LevelSelect { ref mut selected } => {
                // --- INPUTS ---
                let left = rl.is_key_pressed(KeyboardKey::KEY_LEFT);
                let right = rl.is_key_pressed(KeyboardKey::KEY_RIGHT);
                let enter = rl.is_key_pressed(KeyboardKey::KEY_ENTER);
                let esc   = rl.is_key_pressed(KeyboardKey::KEY_ESCAPE);

                if left { *selected = selected.saturating_sub(1); }
                if right { *selected = (*selected + 1).min(2); }

                // --- RENDER ---
                draw_level_select(&mut framebuffer, &tex_manager, *selected, progress.unlocked);
                unsafe {
                    let len = (framebuffer.width * framebuffer.height * 4) as usize;
                    let slice = std::slice::from_raw_parts(
                        framebuffer.color_buffer.data as *const u8, len);
                    screen_tex.update_texture(slice).unwrap();
                }
                {
                    let mut d = rl.begin_drawing(&thread);
                    d.clear_background(Color::BLACK);
                    d.draw_texture(&screen_tex, 0, 0, Color::WHITE);

                    menu::draw_level_select_header_text(&mut d, screen_w, 36);

                    let msg = "ENTER: jugar | ESC: salir";
                    let fs = 24;
                    let tw = d.measure_text(msg, fs);
                    d.draw_text(msg, (screen_w - tw)/2, screen_h - fs - 16, fs, Color::RAYWHITE);
                } // drop d

                // --- TRANSICIONES ---
                if enter {
                    if progress.unlocked[*selected as usize] {
                        current_theme = theme_for(*selected);
                        tex_manager.apply_theme(&mut rl, &thread, &current_theme);
                        audio.load_theme_music(&current_theme);
                        hud.apply_theme(&current_theme);

                        maze_original = load_maze_for_level(*selected);
                        maze = maze_original.clone();

                        // Recalcular spawns iniciales:
                        enemy_spawn_cells.clear();
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

                        keys_sprites = sprites::collect_keys(&maze, block_size, &tex_manager);
                        for row in maze.iter_mut() {
                            for c in row.iter_mut() {
                                if *c == '1' || *c == '2' || *c == '3' { *c = ' '; }
                            }
                        }

                        progress.last_level = *selected;
                        save_progress(&progress);

                        app_state = AppState::InGame { level: *selected };
                        rl.hide_cursor();

                        do_reset(
                            &mut player, &mut enemies, &mut gm, &mut time_s, &mut hud,
                            &mut maze, &mut keys_sprites, &mut enemy_spawn_cells, &mut audio,
                            &tex_manager, &maze_original, block_size,
                            (player_spawn_px.0, player_spawn_px.1),
                            player_spawn_angle, player_spawn_fov,
                        );
                        prev_state = gm.state;
                    } else {
                        // opcional: SFX de error
                    }
                }
                if esc { break; }
            }

            AppState::InGame { level } => {
                // --- INPUTS previos a dibujo ---
                let want_reset = (gm.is_game_over() || gm.is_win()) && rl.is_key_pressed(KeyboardKey::KEY_R);
                let press_e    = rl.is_key_pressed(KeyboardKey::KEY_E);

                match gm.state {
                    GameState::Playing | GameState::JumpScare => rl.hide_cursor(),
                    GameState::GameOver | GameState::Win => rl.show_cursor(),
                }

                if gm.is_playing() {
                    crate::controller::process_events(&mut rl, &mut player, dt, screen_w, screen_h);

                    let player_radius = (block_size as f32) * 0.20;
                    resolve_player_collisions(&mut player.pos, player_radius, &maze, block_size, 2);

                    for e in &mut enemies { update_enemy(e, &maze, player.pos, block_size, dt); }
                }

                let enemy_positions = enemies.iter().map(|e| e.pos);
                gm.update(player.pos, enemy_positions, dt);

                // Audio 3D enemigo + mix
                let max_hear = (block_size as f32) * 6.0;
                audio.update_enemy_proximity(
                    player.pos,
                    player.a,
                    enemies.iter().map(|e| e.pos),
                    max_hear,
                );
                audio.update(dt);

                // Cambio de estado → música/SFX
                if gm.state != prev_state {
                    audio.on_state_changed(gm.state);
                    match gm.state {
                        GameState::Playing => {
                            audio.switch_music("game", false);
                        }
                        GameState::JumpScare => {
                            audio.switch_music("jumpscare", true);
                            audio.play_sfx("jumpscare", 1.0);
                        }
                        GameState::GameOver => {
                            audio.switch_music("gameover", true);
                        }
                        GameState::Win => {
                            audio.switch_music("win", true);
                        }
                    }
                    prev_state = gm.state;
                }

                if gm.is_playing() { hud.update(dt); }

                // Pick-up llaves
                if gm.is_playing() {
                    let pick_radius = (block_size as f32) * 0.45;
                    let pick_r2 = pick_radius * pick_radius;
                    keys_sprites.retain(|s| {
                        let dx = s.pos.x - player.pos.x;
                        let dy = s.pos.y - player.pos.y;
                        let d2 = dx*dx + dy*dy;
                        if d2 <= pick_r2 {
                            audio.play_sfx("key_pick", 1.0);
                            match s.tex {
                                '1' => player.inv.key_yellow = true,
                                '2' => player.inv.key_blue   = true,
                                '3' => player.inv.key_red    = true,
                                _ => {}
                            }
                            false
                        } else { true }
                    });
                }

                // Interacción con puertas (E)
                if press_e && gm.is_playing() {
                    let (ci, cj) = forward_cell(player.pos, player.a, block_size, block_size as f32 * 0.6);
                    if cj >= 0 && (cj as usize) < maze.len() && ci >= 0 && (ci as usize) < maze[cj as usize].len() {
                        let cell = maze[cj as usize][ci as usize];
                        match cell {
                            'Y' if player.inv.key_yellow => {
                                maze[cj as usize][ci as usize] = ' ';
                                audio.play_sfx("door_open", 0.9);
                            }
                            'B' if player.inv.key_blue   => {
                                maze[cj as usize][ci as usize] = ' ';
                                audio.play_sfx("door_open", 0.9);
                            }
                            'R' if player.inv.key_red    => {
                                maze[cj as usize][ci as usize] = ' ';
                                audio.play_sfx("door_open", 0.9);
                            }
                            'G' => {
                                if player.inv.has_all() {
                                    gm.state = GameState::Win;
                                }
                            }
                            _ => {}
                        }
                    }
                }

                // Viewport 3D (excluye HUD)
                let vp_y0 = 0;
                let vp_h  = framebuffer.height - hud.height;

                // --- RENDER ---
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
                            &keys_sprites,
                            time_s,
                            vp_y0,
                            vp_h,
                            &current_theme,
                        );
                        hud.render(&mut framebuffer, &tex_manager, &maze, &player, &enemies, &keys_sprites, block_size);
                    }
                    GameState::JumpScare => {
                        render_world_textured(
                            &mut framebuffer,
                            &maze,
                            &player,
                            block_size,
                            &tex_manager,
                            &sprites,
                            &[],            // enemigos ocultos
                            &keys_sprites,
                            time_s,
                            vp_y0,
                            vp_h,
                            &current_theme,
                        );
                        let fb_w = framebuffer.width;
                        world3d::draw_overlay_viewport(
                            &mut framebuffer, &tex_manager, 'j',
                            0, vp_y0, fb_w, vp_h
                        );
                        hud.render(&mut framebuffer, &tex_manager, &maze, &player, &enemies, &keys_sprites, block_size);
                    }

                    GameState::GameOver => {
                        draw_game_over_background(&mut framebuffer);
                    }

                    GameState::Win => {
                        draw_win_background(&mut framebuffer);
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

                {
                    let mut d = rl.begin_drawing(&thread);
                    d.clear_background(Color::BLACK);
                    d.draw_texture(&screen_tex, 0, 0, Color::WHITE);

                    match gm.state {
                        GameState::Playing => {
                            d.draw_text("E: Abrir puerta", 10, 10, 18, Color::RAYWHITE);
                        }
                        GameState::JumpScare => { /* overlay tapa */ }
                        GameState::GameOver => {
                            let text = "GAME OVER";
                            let font_size = 100;
                            let tw = d.measure_text(text, font_size);
                            let x = (screen_w - tw) / 2;
                            let y = (screen_h - font_size) / 2;
                            d.draw_text(text, x, y, font_size, Color::RED);

                            let sub = "ENTER: menú de niveles  |  R: reiniciar  |  ESC: salir";
                            let sub_size = 24;
                            let sw = d.measure_text(sub, sub_size);
                            let sx = (screen_w - sw) / 2;
                            d.draw_text(sub, sx, y + font_size + 20, sub_size, Color::RAYWHITE);
                        }
                        GameState::Win => {
                            let text = "YOU ESCAPED!";
                            let font_size = 84;
                            let tw = d.measure_text(text, font_size);
                            let x = (screen_w - tw) / 2;
                            let y = (screen_h - font_size) / 2;
                            d.draw_text(text, x, y, font_size, Color::LIME);

                            let sub = "ENTER: menú de niveles  |  R: reiniciar  |  ESC: salir";
                            let sub_size = 24;
                            let sw = d.measure_text(sub, sub_size);
                            let sx = (screen_w - sw) / 2;
                            d.draw_text(sub, sx, y + font_size + 20, sub_size, Color::RAYWHITE);
                        }
                    }
                } // drop d

                // --- TRANSICIONES post-render ---
                let go_menu = (gm.is_game_over() || gm.is_win()) && rl.is_key_pressed(KeyboardKey::KEY_ENTER);
                if want_reset {
                    do_reset(
                        &mut player, &mut enemies, &mut gm, &mut time_s, &mut hud,
                        &mut maze, &mut keys_sprites, &mut enemy_spawn_cells, &mut audio,
                        &tex_manager, &maze_original, block_size,
                        (player_spawn_px.0, player_spawn_px.1),
                        player_spawn_angle, player_spawn_fov,
                    );
                } else if go_menu {
                    if gm.is_win() {
                        if (level as usize) < 2 && !progress.unlocked[level as usize + 1] {
                            progress.unlocked[level as usize + 1] = true;
                            save_progress(&progress);
                        }
                    }
                    app_state = AppState::LevelSelect { selected: level.min(2) };
                    rl.show_cursor();
                }
            }
        }
    }
}

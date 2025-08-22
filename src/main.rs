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
mod physics; // ⬅️ nuevo

use raylib::prelude::*;
use raylib::consts::TextureFilter;

use framebuffer::Framebuffer;
use maze::{load_maze, find_char, maze_dims, Maze};
use player::Player;
use world3d::{render_world_textured, draw_overlay_fullscreen, draw_game_over_background, draw_win_background};
use textures::TextureManager;
use sprites::{collect_sprites, Sprite};
use enemy::{Enemy, update_enemy};
use gamemanager::{GameManager, GameState};
use hud::Hud;
use physics::resolve_player_collisions; // ⬅️ nuevo

fn recreate_enemies(cells: &[(i32,i32)], block_size: usize) -> Vec<Enemy> {
    let mut v = Vec::with_capacity(cells.len());
    for &(ci, cj) in cells { v.push(Enemy::from_cell(ci, cj, block_size)); }
    v
}

// Celda hacia adelante (para interactuar con puertas)
fn forward_cell(pos: Vector2, ang: f32, block_size: usize, steps_px: f32) -> (i32,i32) {
    let nx = pos.x + ang.cos() * steps_px;
    let ny = pos.y + ang.sin() * steps_px;
    let ci = (nx / block_size as f32).floor() as i32;
    let cj = (ny / block_size as f32).floor() as i32;
    (ci, cj)
}

fn main() {
    let screen_w = 1000;
    let screen_h = 800;
    let (mut rl, thread) = raylib::init()
        .size(screen_w, screen_h)
        .title("Raycasting con HUD + Minimap + Keys")
        .build();

    // ⬇️ Capturar/ocultar cursor para estilo FPS (lo liberamos en GameOver/Win)
    rl.set_target_fps(120);   // opcional, pero ayuda
    rl.hide_cursor();  

    let mut framebuffer = Framebuffer::new(screen_w, screen_h, Color::BLACK);
    let tex_manager = TextureManager::new(&mut rl, &thread);

    // 🆕 Cargamos y guardamos una copia prístina del mapa
    let maze_original: Maze = load_maze("assets/maze.txt");
    let mut maze: Maze = maze_original.clone(); // 🔁 trabajamos sobre una copia

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

    // Spawns de enemigos desde el mapa actual
    let mut enemy_spawn_cells: Vec<(i32,i32)> = Vec::new();
    for (j,row) in maze.iter().enumerate() {
        for (i,&c) in row.iter().enumerate() {
            if c == 'e' { enemy_spawn_cells.push((i as i32, j as i32)); }
        }
    }
    // Limpia marcadores 'e' del mapa
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

    // Sprites decorativos (si usas 'e' como decorativo animado)
    let sprites: Vec<Sprite> = collect_sprites(&maze, block_size, &tex_manager);

    // --- LLAVES: recolecta '1','2','3' como sprites y limpia del mapa
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

    // 🆕 reset helper: ahora también restaura puertas (maze) y llaves
    let mut do_reset = |player: &mut Player,
                        enemies: &mut Vec<Enemy>,
                        gm: &mut GameManager,
                        time_s: &mut f32,
                        hud: &mut Hud,
                        maze_ref: &mut Maze,                 // ⬅️ NUEVO
                        keys_ref: &mut Vec<Sprite>,          // ⬅️ NUEVO
                        enemy_cells_ref: &mut Vec<(i32,i32)> // ⬅️ NUEVO
    | {
        // Restaurar laberinto desde copia prístina (vuelven puertas cerradas)
        *maze_ref = maze_original.clone();

        // Recalcular spawns de enemigos desde el mapa restaurado
        enemy_cells_ref.clear();
        for (j,row) in maze_ref.iter().enumerate() {
            for (i,&c) in row.iter().enumerate() {
                if c == 'e' { enemy_cells_ref.push((i as i32, j as i32)); }
            }
        }
        // Limpiar 'e' del mapa (no son paredes)
        for row in maze_ref.iter_mut() {
            for c in row.iter_mut() {
                if *c == 'e' { *c = ' '; }
            }
        }

        // Regenerar llaves segun el mapa restaurado y limpiarlas del mapa
        *keys_ref = sprites::collect_keys(maze_ref, block_size, &tex_manager);
        for row in maze_ref.iter_mut() {
            for c in row.iter_mut() {
                if *c == '1' || *c == '2' || *c == '3' { *c = ' '; }
            }
        }

        // Reset jugador
        player.pos.x = player_spawn_px.0;
        player.pos.y = player_spawn_px.1;
        player.a     = player_spawn_angle;
        player.fov   = player_spawn_fov;
        player.inv.key_yellow = false;
        player.inv.key_blue   = false;
        player.inv.key_red    = false;

        // Reset enemigos
        *enemies = recreate_enemies(enemy_cells_ref, block_size);

        // Reset tiempo/estado/HUD
        *time_s = 0.0;
        gm.reset();
        hud.face_playing = false;
        hud.face_time = 0.0;
        hud.face_cooldown = 1.5;
    };

    while !rl.window_should_close() {        
        let dt = rl.get_frame_time();
        time_s += dt;

        if (gm.is_game_over() || gm.is_win()) && rl.is_key_pressed(KeyboardKey::KEY_R) {
            // 🔁 ahora también reinicia puertas y llaves
            do_reset(&mut player, &mut enemies, &mut gm, &mut time_s, &mut hud,
                     &mut maze, &mut keys_sprites, &mut enemy_spawn_cells);
        }

        // Capturar/soltar cursor según estado
        match gm.state {
            GameState::Playing | GameState::JumpScare => rl.hide_cursor(),
            GameState::GameOver | GameState::Win => rl.show_cursor(),
        }

        if gm.is_playing() {
            // ⬇️ NUEVO: controller con dt y mouse-look
            crate::controller::process_events(&mut rl, &mut player, dt, screen_w, screen_h);

            // ⬇️ Colisiones contra paredes/puertas
            let player_radius = (block_size as f32) * 0.20;
            resolve_player_collisions(&mut player.pos, player_radius, &maze, block_size, 2);

            for e in &mut enemies { update_enemy(e, &maze, player.pos, block_size, dt); }
        }

        let enemy_positions = enemies.iter().map(|e| e.pos);
        gm.update(player.pos, enemy_positions, dt);

        if gm.is_playing() { hud.update(dt); }

        // --- Pick-up de llaves (cerca del jugador)
        if gm.is_playing() {
            let pick_radius = (block_size as f32) * 0.45;
            let pick_r2 = pick_radius * pick_radius;
            keys_sprites.retain(|s| {
                let dx = s.pos.x - player.pos.x;
                let dy = s.pos.y - player.pos.y;
                let d2 = dx*dx + dy*dy;
                if d2 <= pick_r2 {
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

        // --- Interacción con puertas (E)
        if rl.is_key_pressed(KeyboardKey::KEY_E) && gm.is_playing() {
            let (ci, cj) = forward_cell(player.pos, player.a, block_size, block_size as f32 * 0.6);
            if cj >= 0 && (cj as usize) < maze.len() && ci >= 0 && (ci as usize) < maze[cj as usize].len() {
                let cell = maze[cj as usize][ci as usize];
                match cell {
                    'Y' if player.inv.key_yellow => { maze[cj as usize][ci as usize] = ' '; }
                    'B' if player.inv.key_blue   => { maze[cj as usize][ci as usize] = ' '; }
                    'R' if player.inv.key_red    => { maze[cj as usize][ci as usize] = ' '; }
                    'G' => {
                        if player.inv.has_all() {
                            gm.state = GameState::Win;
                        }
                    }
                    _ => {}
                }
            }
        }

        // Viewport del mundo 3D (excluye el HUD)
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
                    &keys_sprites, // 🔑 llaves con oclusión real
                    time_s,
                    vp_y0,
                    vp_h,
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
                    &[],            // enemigos “ocultos” bajo overlay
                    &keys_sprites,  // llaves también renderizadas debajo del overlay
                    time_s,
                    vp_y0,
                    vp_h,
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

        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::BLACK);
        d.draw_texture(&screen_tex, 0, 0, Color::WHITE);

        match gm.state {
            GameState::Playing => {
                d.draw_text("R: Reset | E: Abrir puerta", 10, 10, 18, Color::RAYWHITE);
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
            GameState::Win => {
                let text = "YOU ESCAPED!";
                let font_size = 84;
                let tw = d.measure_text(text, font_size);
                let x = (screen_w - tw) / 2;
                let y = (screen_h - font_size) / 2;
                d.draw_text(text, x, y, font_size, Color::LIME);

                let sub = "Presiona R para reiniciar - ESC para salir";
                let sub_size = 24;
                let sw = d.measure_text(sub, sub_size);
                let sx = (screen_w - sw) / 2;
                d.draw_text(sub, sx, y + font_size + 20, sub_size, Color::RAYWHITE);
            }
        }
    }
}

use raylib::color::Color;
use crate::renderer::MinimapColors;

#[derive(Clone)]
pub struct Lighting {
    pub shade_min: f32, // mínimo multiplicador de luz (0..1)
    pub atten: f32,     // atenuación por distancia (recomendado ~0.001..0.003)
}

#[derive(Clone)]
pub struct LevelTheme {
    // PNGs (por nivel) para walls y personajes/overlays:
    pub wall1: &'static str, // '+', '-', '|'
    pub wall2: &'static str, // '@'
    pub wall3: &'static str, // '#'
    pub wall4: &'static str, // '!'
    pub enemy: &'static str, // 'e' (spritesheet o imagen)
    pub jumps: &'static str, // 'j' overlay

    // Música
    pub music_game: &'static str,
    pub music_jump: &'static str,
    pub music_go:   &'static str,
    pub music_win:  &'static str,
    pub enemy_loop: &'static str,

    // Cielo/suelo
    pub sky:   Color,
    pub floor: Color,

    // Sombreado del mundo y sprites
    pub lighting: Lighting,

    // UI imágenes menú
    pub img_logo:  &'static str, // 'O'
    pub img_card1: &'static str, // 'A'
    pub img_card2: &'static str, // 'B'
    pub img_card3: &'static str, // 'C'
    pub img_lock:  &'static str, // 'K'

    pub mini_wall1: Color, // para '+', '-', '|'
    pub mini_wall2: Color, // para '@'
    pub mini_wall3: Color, // para '#'
    pub mini_wall4: Color, // para '!'
    pub mini_empty: Color, // “suelo/empty”
}

pub fn minimap_colors_for(theme: &LevelTheme) -> MinimapColors {
    let c = |C: Color| (C.r, C.g, C.b, C.a);
    MinimapColors {
        wall1:   c(theme.mini_wall1),
        wall2:   c(theme.mini_wall2),
        wall3:   c(theme.mini_wall3),
        wall4:   c(theme.mini_wall4),
        empty:   c(theme.mini_empty),

        // lo demás se deja “general” (valores por defecto razonables):
        goal:     (200, 60, 60, 255),
        player:   (34, 190, 34, 255),
        enemy:    (200, 40, 40, 255),
        dir_line: (180, 255, 180, 255),
        fov_ray:  (110, 160, 255, 160),
        frame:    (0, 0, 0, 160),
        key_y:    (255, 215, 0, 255),
        key_b:    (60, 130, 255, 255),
        key_r:    (235, 60, 60, 255),
    }
}

pub fn theme_for(level: u8) -> LevelTheme {
    match level {
        0 => LevelTheme {
            wall1: "assets/level1/wall1.png",
            wall2: "assets/level1/wall2.png",
            wall3: "assets/level1/wall3.png",
            wall4: "assets/level1/wall4.png",
            enemy: "assets/level1/enemy.png",
            jumps: "assets/level1/jumpscare.png",

            music_game: "assets/level1/music_gameplay.ogg",
            music_jump: "assets/level1/music_jumpscare.ogg",
            music_go:   "assets/level1/music_gameover.ogg",
            music_win:  "assets/level1/music_win.ogg",
            enemy_loop: "assets/level1/enemy_loop.ogg",

            sky:   Color::new(150, 142, 59, 255),
            floor: Color::new(133, 111, 27, 255),

            lighting: Lighting { shade_min: 0.70, atten: 0.001 },

            img_logo:  "assets/ui/logo.png",
            img_card1: "assets/ui/card_level1.png",
            img_card2: "assets/ui/card_level2.png",
            img_card3: "assets/ui/card_level3.png",
            img_lock:  "assets/ui/lock.png",

            mini_wall1: Color::new(182, 180, 97, 255),
            mini_wall2: Color::new(170, 160, 80, 255),
            mini_wall3: Color::new(150, 140, 70, 255),
            mini_wall4: Color::new(120, 110, 55, 255),
            mini_empty: Color::new(150, 142, 59, 255),
        },
        1 => LevelTheme {
            wall1: "assets/level2/wall1.png",
            wall2: "assets/level2/wall2.png",
            wall3: "assets/level2/wall3.png",
            wall4: "assets/level2/wall4.png",
            enemy: "assets/level2/enemy.png",
            jumps: "assets/level2/jumpscare.png",

            music_game: "assets/level2/music_gameplay.ogg",
            music_jump: "assets/level2/music_jumpscare.ogg",
            music_go:   "assets/level2/music_gameover.ogg",
            music_win:  "assets/level2/music_win.ogg",
            enemy_loop: "assets/level2/enemy_loop.ogg",

            sky:   Color::new(79, 79, 79, 255),
            floor: Color::new(0, 128, 75, 255),

            lighting: Lighting { shade_min: 0.65, atten: 0.0015 },

            img_logo:  "assets/ui/logo.png",
            img_card1: "assets/ui/card_level1.png",
            img_card2: "assets/ui/card_level2.png",
            img_card3: "assets/ui/card_level3.png",
            img_lock:  "assets/ui/lock.png",

            mini_wall1: Color::new(120, 120, 120, 255),
            mini_wall2: Color::new(105, 105, 105, 255),
            mini_wall3: Color::new(95, 95, 95, 255),
            mini_wall4: Color::new(80, 80, 80, 255),
            mini_empty: Color::new(60, 110, 80, 255),
        },
        _ => LevelTheme {
            wall1: "assets/level3/wall1.png",
            wall2: "assets/level3/wall2.png",
            wall3: "assets/level3/wall3.png",
            wall4: "assets/level3/wall4.png",
            enemy: "assets/level3/enemy.png",
            jumps: "assets/level3/jumpscare.png",

            music_game: "assets/level3/music_gameplay.ogg",
            music_jump: "assets/level3/music_jumpscare.ogg",
            music_go:   "assets/level3/music_gameover.ogg",
            music_win:  "assets/level3/music_win.ogg",
            enemy_loop: "assets/level3/enemy_loop.ogg",

            sky:   Color::new(0, 40, 112, 255),
            floor: Color::new(143, 143, 143, 255),

            lighting: Lighting { shade_min: 0.01, atten: 0.0070 },

            img_logo:  "assets/ui/logo.png",
            img_card1: "assets/ui/card_level1.png",
            img_card2: "assets/ui/card_level2.png",
            img_card3: "assets/ui/card_level3.png",
            img_lock:  "assets/ui/lock.png",

            mini_wall1: Color::new(180, 60, 60, 255),
            mini_wall2: Color::new(200, 90, 70, 255),
            mini_wall3: Color::new(220, 110, 80, 255),
            mini_wall4: Color::new(240, 140, 90, 255),
            mini_empty: Color::new(90, 90, 90, 255),
        },
    }
}

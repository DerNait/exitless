use std::collections::HashMap;

use raylib::core::audio::{Music, RaylibAudio, Sound};
use raylib::prelude::Vector2;

use crate::gamemanager::GameState;
use crate::level::LevelTheme;

/// Claves para identificar pistas
const K_GAME: &str = "game";
const K_JUMP: &str = "jumpscare";
const K_GO:   &str = "gameover";
const K_WIN:  &str = "win";
const K_ENEM: &str = "enemy_loop";

/// Config sencilla de audio (ajusta a tu gusto).
#[derive(Clone, Copy)]
pub struct AudioConfig {
    // Masters
    pub master_music: f32,   // 0..1
    pub master_sfx:   f32,   // 0..1

    // Volumen base de cada música (antes de crossfade)
    pub vol_music_game:      f32,
    pub vol_music_jumpscare: f32,
    pub vol_music_gameover:  f32,
    pub vol_music_win:       f32,

    // Loop enemigo (base). Se multiplica por la ganancia por distancia.
    pub vol_enemy_loop: f32,

    // SFX
    pub vol_sfx_door_open: f32,
    pub vol_sfx_key_pick:  f32,
    pub vol_sfx_jumpscare: f32,

    // Crossfade
    pub fade_speed: f32,      // unidades por segundo; 2.0 ≈ 0.5s de fade

    // Audio “3D”
    pub enemy_max_dist: f32,  // px de radio máximo de escucha

    // Reglas de en qué estados se permite el loop del enemigo
    pub enemy_loop_in_playing:   bool,
    pub enemy_loop_in_jumpscare: bool,
    pub enemy_loop_in_gameover:  bool,
    pub enemy_loop_in_win:       bool,
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            master_music: 1.0,
            master_sfx:   1.0,

            vol_music_game:      0.8,
            vol_music_jumpscare: 1.0,
            vol_music_gameover:  0.9,
            vol_music_win:       0.9,

            vol_enemy_loop: 0.9,

            vol_sfx_door_open: 0.9,
            vol_sfx_key_pick:  1.0,
            vol_sfx_jumpscare: 1.0,

            fade_speed: 2.0,

            enemy_max_dist: 600.0,

            enemy_loop_in_playing:   true,
            enemy_loop_in_jumpscare: false,
            enemy_loop_in_gameover:  false,
            enemy_loop_in_win:       false,
        }
    }
}

impl AudioConfig {
    #[inline]
    fn music_base(&self, key: &str) -> f32 {
        match key {
            K_GAME => self.vol_music_game,
            K_JUMP => self.vol_music_jumpscare,
            K_GO   => self.vol_music_gameover,
            K_WIN  => self.vol_music_win,
            K_ENEM => self.vol_enemy_loop,
            _ => 1.0,
        }
    }

    #[inline]
    fn sfx_base(&self, key: &str) -> f32 {
        match key {
            "door_open" => self.vol_sfx_door_open,
            "key_pick"  => self.vol_sfx_key_pick,
            "jumpscare" => self.vol_sfx_jumpscare,
            _ => 1.0,
        }
    }

    #[inline]
    fn enemy_loop_allowed_in(&self, st: GameState) -> bool {
        match st {
            GameState::Playing   => self.enemy_loop_in_playing,
            GameState::JumpScare => self.enemy_loop_in_jumpscare,
            GameState::GameOver  => self.enemy_loop_in_gameover,
            GameState::Win       => self.enemy_loop_in_win,
        }
    }
}

/// Gestor de audio con:
/// - Músicas por estado (crossfade desde 0 del nuevo tema)
/// - SFX puntuales
/// - Audio “3D” simple para el enemigo (volumen + paneo por cercanía/lado)
pub struct AudioManager<'a> {
    ra: &'a RaylibAudio,

    musics: HashMap<&'static str, Music<'a>>,
    sfx:    HashMap<&'static str, Sound<'a>>,

    // Crossfade
    current_key: Option<&'static str>,
    target_key:  Option<&'static str>,
    fade_t: f32, // 0..1

    // Enemy loop controlado aparte
    enemy_gain: f32,     // 0..1 (calculado por distancia)
    enemy_max_dist: f32, // px

    // Config
    cfg: AudioConfig,

    // Estado actual (para reglas)
    state: GameState,
}

impl<'a> AudioManager<'a> {
    pub fn new_default(ra: &'a RaylibAudio) -> Self {
        Self::new(ra, AudioConfig::default())
    }

    pub fn new(ra: &'a RaylibAudio, cfg: AudioConfig) -> Self {
        let mut musics = HashMap::new();
        let mut sfx    = HashMap::new();

        // Cargamos *placeholder* iniciales (se reemplazan con load_theme_music)
        musics.insert(K_GAME, ra.new_music("assets/audio/music_gameplay.ogg").expect("music_gameplay.ogg"));
        musics.insert(K_JUMP, ra.new_music("assets/audio/music_jumpscare.ogg").expect("music_jumpscare.ogg"));
        musics.insert(K_GO,   ra.new_music("assets/audio/music_gameover.ogg").expect("music_gameover.ogg"));
        musics.insert(K_WIN,  ra.new_music("assets/audio/music_win.ogg").expect("music_win.ogg"));
        musics.insert(K_ENEM, ra.new_music("assets/audio/enemy_loop.ogg").expect("enemy_loop.ogg"));

        // --- Cargar SFX ---
        sfx.insert("door_open", ra.new_sound("assets/audio/sfx_door_open.wav").expect("sfx_door_open.wav"));
        sfx.insert("key_pick",  ra.new_sound("assets/audio/sfx_key_pick.wav").expect("sfx_key_pick.wav"));
        sfx.insert("jumpscare", ra.new_sound("assets/audio/sfx_jumpscare.wav").expect("sfx_jumpscare.wav"));

        let mut this = Self {
            ra,
            musics,
            sfx,
            current_key: None,
            target_key:  None,
            fade_t: 1.0,
            enemy_gain: 0.0,
            enemy_max_dist: cfg.enemy_max_dist,
            cfg,
            state: GameState::Playing,
        };

        // Arrancamos gameplay desde 0 y paramos loop enemigo
        this.switch_music(K_GAME, true);
        this.stop_music(K_ENEM);

        this
    }

    /// Reemplaza fuentes musicales según el tema (nivel).
    pub fn load_theme_music(&mut self, theme: &LevelTheme) {
        self.musics.insert(K_GAME, self.ra.new_music(theme.music_game).expect("music_game"));
        self.musics.insert(K_JUMP, self.ra.new_music(theme.music_jump).expect("music_jump"));
        self.musics.insert(K_GO,   self.ra.new_music(theme.music_go).expect("music_go"));
        self.musics.insert(K_WIN,  self.ra.new_music(theme.music_win).expect("music_win"));
        self.musics.insert(K_ENEM, self.ra.new_music(theme.enemy_loop).expect("enemy_loop"));

        // Forzamos estado base
        self.reset_to_game();
    }

    /// Cambia la música objetivo. Si `instant` es true, corta la actual y
    /// arranca la nueva DESDE 0; si es false, hace crossfade (nueva arranca desde 0).
    pub fn switch_music(&mut self, key: &'static str, instant: bool) {
        if instant {
            if let Some(cur) = self.current_key {
                if cur != key { self.stop_music(cur); }
            }
            self.start_music_from_zero(key);
            self.current_key = Some(key);
            self.target_key = None;
            self.fade_t = 1.0;
            self.apply_mix_volumes();
            return;
        }

        if self.current_key == Some(key) && self.target_key.is_none() {
            return;
        }

        self.target_key = Some(key);
        self.fade_t = 0.0;
        self.start_music_from_zero(key);
        self.apply_mix_volumes();
    }

    /// Reproduce un SFX por clave. `vol` es adicional (0..1).
    pub fn play_sfx(&mut self, key: &str, vol: f32) {
        if let Some(s) = self.sfx.get(key) {
            let base = self.cfg.sfx_base(key);
            s.set_volume((self.cfg.master_sfx * base * vol).clamp(0.0, 1.0));
            s.play();
        }
    }

    /// Llamar cada frame.
    pub fn update(&mut self, dt: f32) {
        for m in self.musics.values() { m.update_stream(); }

        if let Some(target) = self.target_key {
            self.fade_t += (self.cfg.fade_speed * dt).clamp(0.0, 10.0);
            if self.fade_t >= 1.0 {
                self.fade_t = 1.0;
                if let Some(cur) = self.current_key {
                    if cur != target { self.stop_music(cur); }
                }
                self.current_key = Some(target);
                self.target_key = None;
            }
            self.apply_mix_volumes();
        } else {
            self.apply_mix_volumes();
        }

        if self.cfg.enemy_loop_allowed_in(self.state) {
            if let Some(loop_m) = self.musics.get(K_ENEM) {
                let v = self.cfg.master_music * self.cfg.music_base(K_ENEM) * self.enemy_gain;
                loop_m.set_volume(v.clamp(0.0, 1.0));
            }
        } else {
            self.enemy_gain = 0.0;
            self.stop_music(K_ENEM);
        }
    }

    /// Notifica cambio de estado de juego (para reglas como mutear el enemy loop).
    pub fn on_state_changed(&mut self, st: GameState) {
        self.state = st;
        if !self.cfg.enemy_loop_allowed_in(st) {
            self.enemy_gain = 0.0;
            self.stop_music(K_ENEM);
        }
    }

    /// Ajusta volumen/pan del loop del enemigo según distancia y lado.
    pub fn update_enemy_proximity<I: IntoIterator<Item = Vector2>>(
        &mut self,
        player_pos: Vector2,
        player_dir: f32,
        enemy_positions: I,
        max_hear_dist: f32,
    ) {
        self.enemy_max_dist = max_hear_dist.max(1.0);

        if !self.cfg.enemy_loop_allowed_in(self.state) {
            self.enemy_gain = 0.0;
            return;
        }

        let mut best_d2 = f32::INFINITY;
        let mut best_vec = Vector2::new(0.0, 0.0);
        for epos in enemy_positions {
            let dx = epos.x - player_pos.x;
            let dy = epos.y - player_pos.y;
            let d2 = dx*dx + dy*dy;
            if d2 < best_d2 { best_d2 = d2; best_vec = Vector2::new(dx, dy); }
        }

        if !best_d2.is_finite() { self.enemy_gain = 0.0; return; }

        let d = best_d2.sqrt();
        let raw = (1.0 - (d / self.enemy_max_dist)).clamp(0.0, 1.0);
        self.enemy_gain = raw * raw;

        let fx = player_dir.cos();
        let fy = player_dir.sin();
        let len = (best_vec.x*best_vec.x + best_vec.y*best_vec.y).sqrt().max(1e-6);
        let tx = best_vec.x / len;
        let ty = best_vec.y / len;
        let cross = fx * ty - fy * tx; // [-1,1]
        let pan01 = (0.5 - 0.5 * cross).clamp(0.0, 1.0);

        if let Some(loop_m) = self.musics.get(K_ENEM) {
            loop_m.set_pan(pan01);
            if self.enemy_gain > 0.0 && !loop_m.is_stream_playing() {
                self.start_music_from_zero(K_ENEM);
            }
        }
    }

    /// Resetea a música de juego desde 0 y detiene el resto (incluyendo enemy loop).
    pub fn reset_to_game(&mut self) {
        self.stop_all();
        self.state = GameState::Playing;
        self.start_music_from_zero(K_GAME);
        self.current_key = Some(K_GAME);
        self.target_key = None;
        self.fade_t = 1.0;
        self.enemy_gain = 0.0;
        self.apply_mix_volumes();
    }

    // ----------------- Internos -----------------
    fn stop_music(&mut self, key: &str) {
        if let Some(m) = self.musics.get(key) {
            m.set_volume(0.0);
            m.stop_stream();
        }
    }

    fn start_music_from_zero(&mut self, key: &str) {
        if let Some(m) = self.musics.get(key) {
            m.stop_stream();
            m.set_volume(0.0);
            m.play_stream();
        }
    }

    fn stop_all(&mut self) {
        for m in self.musics.values() {
            m.set_volume(0.0);
            m.stop_stream();
        }
    }

    fn apply_mix_volumes(&mut self) {
        for (k, m) in self.musics.iter() {
            if *k != K_ENEM { m.set_volume(0.0); }
        }

        let mm = self.cfg.master_music.clamp(0.0, 1.0);

        match (self.current_key, self.target_key) {
            (Some(cur), Some(tgt)) if cur != tgt => {
                if let Some(m) = self.musics.get(cur) {
                    let base = self.cfg.music_base(cur);
                    m.set_volume((mm * base * (1.0 - self.fade_t)).clamp(0.0, 1.0));
                }
                if let Some(m) = self.musics.get(tgt) {
                    let base = self.cfg.music_base(tgt);
                    m.set_volume((mm * base * self.fade_t).clamp(0.0, 1.0));
                }
            }
            (Some(cur), None) => {
                if let Some(m) = self.musics.get(cur) {
                    let base = self.cfg.music_base(cur);
                    m.set_volume((mm * base).clamp(0.0, 1.0));
                }
            }
            (None, Some(tgt)) => {
                if let Some(m) = self.musics.get(tgt) {
                    let base = self.cfg.music_base(tgt);
                    m.set_volume((mm * base * self.fade_t).clamp(0.0, 1.0));
                }
            }
            _ => {}
        }
    }
}

// gamemanager.rs
use raylib::prelude::Vector2;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GameState {
    Playing,
    JumpScare,
    GameOver,
    Win,
}

pub struct GameManager {
    pub state: GameState,
    trigger_dist: f32,
    jumpscare_left: f32,
    jumpscare_total: f32,
}

impl GameManager {
    pub fn new(trigger_dist: f32, jumpscare_seconds: f32) -> Self {
        Self {
            state: GameState::Playing,
            trigger_dist,
            jumpscare_left: 0.0,
            jumpscare_total: jumpscare_seconds,
        }
    }

    pub fn reset(&mut self) {
        self.state = GameState::Playing;
        self.jumpscare_left = 0.0;
    }

    /// Llama esto una vez por frame para avanzar la lógica de estados.
    /// - `player_pos`: posición actual del jugador (px)
    /// - `enemy_positions`: iterator sobre posiciones de enemigos (px)
    /// - `dt`: delta time
    pub fn update<I: IntoIterator<Item = Vector2>>(
        &mut self,
        player_pos: Vector2,
        enemy_positions: I,
        dt: f32,
    ) {
        match self.state {
            GameState::Playing => {
                // Si algún enemigo está a menos de trigger_dist, disparamos JumpScare
                for epos in enemy_positions {
                    let d2 = (epos.x - player_pos.x).powi(2) + (epos.y - player_pos.y).powi(2);
                    if d2 <= self.trigger_dist * self.trigger_dist {
                        self.state = GameState::JumpScare;
                        self.jumpscare_left = self.jumpscare_total;
                        break;
                    }
                }
            }
            GameState::JumpScare => {
                self.jumpscare_left -= dt;
                if self.jumpscare_left <= 0.0 {
                    self.state = GameState::GameOver;
                }
            }
            GameState::GameOver => {
                // (Opcional) Podrías escuchar una tecla para reiniciar aquí.
            }
            GameState::Win => {
                // (Opcional) Podrías escuchar una tecla para reiniciar aquí.
            }
        }
    }

    pub fn is_playing(&self) -> bool { self.state == GameState::Playing }
    pub fn is_jumpscare(&self) -> bool { self.state == GameState::JumpScare }
    pub fn is_game_over(&self) -> bool { self.state == GameState::GameOver }
    pub fn is_win(&self) -> bool { self.state == GameState::Win }
}

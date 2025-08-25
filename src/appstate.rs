#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AppState {
    /// Pantalla de inicio: "Presiona ENTER para comenzar"
    StartScreen,
    /// Selector de nivel; `selected` es 0..2 (niveles 1..3)
    LevelSelect { selected: u8 },
    /// Juego corriendo para un nivel
    InGame { level: u8 },
}

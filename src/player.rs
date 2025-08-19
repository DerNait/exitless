use raylib::prelude::Vector2;

pub struct Player {
    pub pos: Vector2, // en píxeles del framebuffer
    pub a: f32,       // ángulo de vista (radianes)
    pub fov: f32,     // field of view (radianes)
}

impl Player {
    pub fn new(pos: Vector2, a: f32, fov: f32) -> Self {
        Self { pos, a, fov }
    }
}

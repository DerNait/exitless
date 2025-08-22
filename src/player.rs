use raylib::prelude::Vector2;

#[derive(Default, Clone, Copy)]
pub struct Inventory {
    pub key_yellow: bool,
    pub key_blue:   bool,
    pub key_red:    bool,
}

impl Inventory {
    #[inline]
    pub fn has_all(&self) -> bool {
        self.key_yellow && self.key_blue && self.key_red
    }
}

pub struct Player {
    pub pos: Vector2, // en píxeles del framebuffer
    pub a: f32,       // ángulo de vista (radianes)
    pub fov: f32,     // field of view (radianes)
    pub inv: Inventory,
}

impl Player {
    pub fn new(pos: Vector2, a: f32, fov: f32) -> Self {
        Self { pos, a, fov, inv: Inventory::default() }
    }
}

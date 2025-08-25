use raylib::prelude::*;
use raylib::consts::PixelFormat;
use std::collections::HashMap;

use crate::level::LevelTheme;

#[derive(Clone, Copy)]
pub struct TexSheet { pub cols: usize, pub rows: usize, pub frame_w: usize, pub frame_h: usize }

pub struct TexturePixels { pub w: usize, pub h: usize, pub data: Vec<u8> }

pub struct TextureManager {
    pub(crate) pixels: HashMap<char, TexturePixels>,
    pub(crate) _textures: HashMap<char, Texture2D>,
    sheets: HashMap<char, TexSheet>,
}

impl TextureManager {
    pub fn new(rl: &mut RaylibHandle, thread: &RaylibThread) -> Self {
        let mut pixels = HashMap::new();
        let mut textures = HashMap::new();

        // Cargamos un set base (placeholders) que luego podremos sobrescribir por tema
        let texture_files = vec![
            ('+', "assets/wall1.png"),
            ('-', "assets/wall1.png"),
            ('|', "assets/wall1.png"),
            ('@', "assets/wall2.png"),
            ('#', "assets/wall3.png"),
            ('!', "assets/wall4.png"),

            // Enemy & overlays/HUD
            ('e', "assets/enemy1.png"),
            ('j', "assets/jumpscare1.png"),
            ('h', "assets/hud_bg.png"),
            ('f', "assets/face.png"),

            // Keys in world
            ('1', "assets/keys1.png"),
            ('2', "assets/keys2.png"),
            ('3', "assets/keys3.png"),

            // HUD icons (llaves)
            ('y', "assets/keyhud_yellow.png"),
            ('b', "assets/keyhud_blue.png"),
            ('r', "assets/keyhud_red.png"),

            // Doors
            ('Y', "assets/door_yellow.png"),
            ('B', "assets/door_blue.png"),
            ('R', "assets/door_red.png"),
            ('G', "assets/door_exit.png"),

            // FPS digits
            ('n', "assets/fps_digits.png"),

            // UI Menú (se sobrescriben al aplicar tema 1, pero por si acaso)
            ('O', "assets/ui/logo.png"),
            ('A', "assets/ui/card_level1.png"),
            ('N', "assets/ui/card_level2.png"),
            ('C', "assets/ui/card_level3.png"),
            ('K', "assets/ui/lock.png"),
        ];

        for (ch, path) in texture_files {
            let mut image = Image::load_image(path).expect(&format!("load {}", path));
            if image.format() != PixelFormat::PIXELFORMAT_UNCOMPRESSED_R8G8B8A8 {
                image.set_format(PixelFormat::PIXELFORMAT_UNCOMPRESSED_R8G8B8A8);
            }
            let tex = rl.load_texture_from_image(thread, &image).expect("gpu tex");
            let w = image.width as usize;
            let h = image.height as usize;
            let len = w * h * 4;
            let ptr = image.data as *const u8;
            let data = unsafe { std::slice::from_raw_parts(ptr, len) }.to_vec();

            textures.insert(ch, tex);
            pixels.insert(ch, TexturePixels { w, h, data });
        }

        let mut sheets = HashMap::new();

        // Enemy 'e' (ajusta a tu asset real)
        if let Some(p) = pixels.get(&'e') {
            let cols = 4; let rows = 2;
            sheets.insert('e', TexSheet { cols, rows, frame_w: p.w / cols, frame_h: p.h / rows });
        }

        // Face 'f'
        if let Some(p) = pixels.get(&'f') {
            let cols = 4; let rows = 2;
            sheets.insert('f', TexSheet { cols, rows, frame_w: p.w / cols, frame_h: p.h / rows });
        }

        // Keys '1','2','3'
        if let Some(p) = pixels.get(&'1') {
            let cols = 4; let rows = 2;
            sheets.insert('1', TexSheet { cols, rows, frame_w: p.w / cols, frame_h: p.h / rows });
        }
        if let Some(p) = pixels.get(&'2') {
            let cols = 4; let rows = 2;
            sheets.insert('2', TexSheet { cols, rows, frame_w: p.w / cols, frame_h: p.h / rows });
        }
        if let Some(p) = pixels.get(&'3') {
            let cols = 4; let rows = 2;
            sheets.insert('3', TexSheet { cols, rows, frame_w: p.w / cols, frame_h: p.h / rows });
        }

        // FPS digits 'n'
        if let Some(p) = pixels.get(&'n') {
            let cols = 10; let rows = 1;
            sheets.insert('n', TexSheet { cols, rows, frame_w: p.w / cols, frame_h: p.h / rows });
        }

        Self { pixels, _textures: textures, sheets }
    }

    pub fn tex_size(&self, ch: char) -> (u32, u32) {
        self.pixels.get(&ch)
            .or_else(|| self.pixels.get(&'#'))
            .map(|p| (p.w as u32, p.h as u32))
            .unwrap_or((64, 64))
    }

    /// Vista de la imagen completa (w,h,data).
    pub fn tex_view(&self, ch: char) -> (usize, usize, &[u8]) {
        if let Some(p) = self.pixels.get(&ch)       { (p.w, p.h, &p.data) }
        else if let Some(p) = self.pixels.get(&'#') { (p.w, p.h, &p.data) }
        else { (1, 1, &[255, 255, 255, 255][..]) }
    }

    pub fn sheet_meta(&self, ch: char) -> Option<TexSheet> { self.sheets.get(&ch).copied() }

    /// Cantidad total de frames en la hoja (o 1 si no hay hoja).
    pub fn sheet_frames(&self, ch: char) -> usize {
        if let Some(s) = self.sheet_meta(ch) { s.cols * s.rows } else { 1 }
    }

    /// Devuelve metadata para un frame del spritesheet:
    pub fn sheet_frame_view(&self, ch: char, frame: usize)
        -> (usize, usize, usize, usize, usize, usize, &[u8])
    {
        let (tw, th, data) = self.tex_view(ch);
        if let Some(s) = self.sheet_meta(ch) {
            let fx = frame % s.cols;
            let fy = frame / s.cols;
            let x0 = fx * s.frame_w;
            let y0 = fy * s.frame_h;
            return (tw, th, x0, y0, s.frame_w, s.frame_h, data);
        }
        (tw, th, 0, 0, tw, th, data)
    }

    /// Dimensiones de un frame (o de la imagen si no es hoja).
    pub fn frame_dims(&self, ch: char) -> (i32, i32) {
        if let Some(s) = self.sheet_meta(ch) { (s.frame_w as i32, s.frame_h as i32) }
        else {
            let (w,h,_) = self.tex_view(ch);
            (w as i32, h as i32)
        }
    }

    /// Carga/recarga texturas específicas para un tema de nivel.
    pub fn apply_theme(&mut self, rl: &mut RaylibHandle, thread: &RaylibThread, theme: &LevelTheme) {
        let mut load_into = |ch: char, path: &str| {
            let mut image = Image::load_image(path).expect(&format!("load {}", path));
            if image.format() != PixelFormat::PIXELFORMAT_UNCOMPRESSED_R8G8B8A8 {
                image.set_format(PixelFormat::PIXELFORMAT_UNCOMPRESSED_R8G8B8A8);
            }
            let tex = rl.load_texture_from_image(thread, &image).expect("gpu tex");
            let w = image.width as usize;
            let h = image.height as usize;
            let len = w * h * 4;
            let ptr = image.data as *const u8;
            let data = unsafe { std::slice::from_raw_parts(ptr, len) }.to_vec();
            self._textures.insert(ch, tex);
            self.pixels.insert(ch, TexturePixels { w, h, data });
        };

        // Walls:
        load_into('+', theme.wall1);
        load_into('-', theme.wall1);
        load_into('|', theme.wall1);
        load_into('@', theme.wall2);
        load_into('#', theme.wall3);
        load_into('!', theme.wall4);

        // Enemy & Jumpscare overlays:
        load_into('e', theme.enemy);
        load_into('j', theme.jumps);

        // UI (menu):
        load_into('O', theme.img_logo);
        load_into('A', theme.img_card1);
        load_into('N', theme.img_card2);
        load_into('C', theme.img_card3);
        load_into('K', theme.img_lock);

        // Si el spritesheet del enemy cambia de layout, ajusta aquí:
        if let Some(p) = self.pixels.get(&'e') {
            let cols = 4; let rows = 2; // <- cambia si tu hoja difiere
            self.sheets.insert('e', TexSheet { cols, rows, frame_w: p.w / cols, frame_h: p.h / rows });
        }
    }
}

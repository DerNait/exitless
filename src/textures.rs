use raylib::prelude::*;
use raylib::consts::PixelFormat;
use std::collections::HashMap;

#[derive(Clone, Copy)]
pub struct TexSheet { pub cols: usize, pub rows: usize, pub frame_w: usize, pub frame_h: usize }

pub struct TexturePixels { pub w: usize, pub h: usize, pub data: Vec<u8> }

pub struct TextureManager {
    pixels: HashMap<char, TexturePixels>,
    _textures: HashMap<char, Texture2D>,
    sheets: HashMap<char, TexSheet>,
}

impl TextureManager {
    pub fn new(rl: &mut RaylibHandle, thread: &RaylibThread) -> Self {
        let mut pixels = HashMap::new();
        let mut textures = HashMap::new();
        let texture_files = vec![
            ('+', "assets/wall4.png"),
            ('-', "assets/wall2.png"),
            ('|', "assets/wall1.png"),
            ('g', "assets/wall5.png"),
            ('#', "assets/wall3.png"),
            ('e', "assets/enemy1.png"), // spritesheet 4x2 (512x256 => frames 128x128)
            ('j', "assets/jumpscare1.png"),  // overlay para el jumpscare (recomendado con alpha o fondo negro)
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

        // Registra layouts de spritesheets
        let mut sheets = HashMap::new();
        if let Some(p) = pixels.get(&'e') {
            let cols = 4; let rows = 2; // <- AJUSTA AQUÍ si cambias tu hoja
            sheets.insert('e', TexSheet { cols, rows, frame_w: p.w / cols, frame_h: p.h / rows });
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
    /// - tw, th: ancho/alto de la imagen completa
    /// - x0, y0: origen del frame dentro de la imagen
    /// - fw, fh: tamaño del frame
    /// - data: slice RGBA8 de toda la imagen (usaremos x0,y0 para indexar)
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
}

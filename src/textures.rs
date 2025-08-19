use raylib::prelude::*;
use raylib::consts::PixelFormat;
use std::collections::HashMap;

pub struct TexturePixels { pub w: usize, pub h: usize, pub data: Vec<u8> }

pub struct TextureManager {
    pixels: HashMap<char, TexturePixels>,
    _textures: HashMap<char, Texture2D>,
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

        Self { pixels, _textures: textures }
    }

    pub fn tex_size(&self, ch: char) -> (u32, u32) {
        self.pixels.get(&ch)
            .or_else(|| self.pixels.get(&'#'))
            .map(|p| (p.w as u32, p.h as u32))
            .unwrap_or((64, 64))
    }

    /// Devuelve una vista (w,h,data) para usarla dentro del bucle de la columna.
    pub fn tex_view(&self, ch: char) -> (usize, usize, &[u8]) {
        if let Some(p) = self.pixels.get(&ch) {
            (p.w, p.h, &p.data)
        } else if let Some(p) = self.pixels.get(&'#') {
            (p.w, p.h, &p.data)
        } else {
            (1, 1, &[255, 255, 255, 255][..]) // blanco
        }
    }
}

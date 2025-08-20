use raylib::prelude::*;

pub struct Framebuffer {
    pub width: i32,
    pub height: i32,
    pub color_buffer: Image,
    pub background_color: Color,
    pub current_color: Color,
}

impl Framebuffer {
    pub fn new(width: i32, height: i32, background_color: Color) -> Self {
        let color_buffer = Image::gen_image_color(width, height, background_color);
        Self { width, height, color_buffer, background_color, current_color: Color::WHITE }
    }

    /// Si el 3D repinta todo (sky/floor), puedes no llamarla en 3D.
    pub fn clear(&mut self) {
        let (w, h) = (self.width as usize, self.height as usize);
        let len = w * h * 4;
        let c = self.background_color;
        unsafe {
            let base = self.color_buffer.data as *mut u8;
            let mut i = 0;
            while i < len {
                *base.add(i)     = c.r;
                *base.add(i + 1) = c.g;
                *base.add(i + 2) = c.b;
                *base.add(i + 3) = c.a;
                i += 4;
            }
        }
    }

    #[inline]
    pub fn set_pixel(&mut self, x: i32, y: i32) {
        if x < 0 || y < 0 || x >= self.width || y >= self.height { return; }
        let idx = ((y as usize * self.width as usize) + x as usize) * 4;
        unsafe {
            let base = self.color_buffer.data as *mut u8;
            *base.add(idx)     = self.current_color.r;
            *base.add(idx + 1) = self.current_color.g;
            *base.add(idx + 2) = self.current_color.b;
            *base.add(idx + 3) = self.current_color.a;
        }
    }

    #[inline]
    pub fn put_pixel_rgba(&mut self, x: i32, y: i32, r: u8, g: u8, b: u8, a: u8) {
        if x < 0 || y < 0 || x >= self.width || y >= self.height { return; }
        let idx = ((y as usize * self.width as usize) + x as usize) * 4;
        unsafe {
            let base = self.color_buffer.data as *mut u8;
            *base.add(idx)     = r;
            *base.add(idx + 1) = g;
            *base.add(idx + 2) = b;
            *base.add(idx + 3) = a;
        }
    }

    // Opcional: single 32-bit store (según layout RGBA8)
    #[inline]
    pub fn put_pixel_rgba_u32(&mut self, x: i32, y: i32, rgba: u32) {
        if x < 0 || y < 0 || x >= self.width || y >= self.height { return; }
        let idx = ((y as usize * self.width as usize) + x as usize);
        unsafe {
            let base = self.color_buffer.data as *mut u32;
            *base.add(idx) = rgba;
        }
    }

    #[inline]
    pub fn fill_row(&mut self, y: i32, color: Color) {
        if y < 0 || y >= self.height { return; }
        let w = self.width as usize;
        let start = (y as usize * w) * 4;
        unsafe {
            let base = self.color_buffer.data as *mut u8;
            let mut i = 0usize;
            while i < w {
                let p = start + i * 4;
                *base.add(p)     = color.r;
                *base.add(p + 1) = color.g;
                *base.add(p + 2) = color.b;
                *base.add(p + 3) = color.a;
                i += 1;
            }
        }
    }

    pub fn set_background_color(&mut self, color: Color) { self.background_color = color; }
    pub fn set_current_color(&mut self, color: Color)     { self.current_color = color; }

    pub fn render_to_file(&self, file_path: &str) { Image::export_image(&self.color_buffer, file_path); }
}

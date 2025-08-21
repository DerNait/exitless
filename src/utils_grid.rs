use raylib::prelude::Vector2;

#[inline]
pub fn world_to_cell(x: f32, y: f32, bs: usize) -> (i32,i32) {
    let bsf = bs as f32;
    ((x/bsf).floor() as i32, (y/bsf).floor() as i32)
}

#[inline]
pub fn cell_center(ci: i32, cj: i32, bs: usize) -> Vector2 {
    let bsf = bs as f32;
    Vector2::new(ci as f32 * bsf + bsf*0.5, cj as f32 * bsf + bsf*0.5)
}

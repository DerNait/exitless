use raylib::color::Color;
use crate::framebuffer::Framebuffer;
use crate::maze::Maze;

pub fn draw_cell(
    framebuffer: &mut Framebuffer,
    xo: usize,
    yo: usize,
    block_size: usize,
    cell: char,
) {
    let color = match cell {
        '+' | '-' | '|' => Color::DARKGRAY,  // paredes
        'p' => Color::GREEN,                  // player start
        'g' => Color::RED,                    // goal
        _   => Color::BLANK,                  // espacios
    };
    framebuffer.set_current_color(color);

    // pequeño “fill rect”
    for y in 0..block_size {
        for x in 0..block_size {
            framebuffer.set_pixel((xo + x) as i32, (yo + y) as i32);
        }
    }
}

pub fn render_maze(
    framebuffer: &mut Framebuffer,
    maze: &Maze,
    block_size: usize,
) {
    for (row_index, row) in maze.iter().enumerate() {
        for (col_index, cell) in row.iter().enumerate() {
            let xo = col_index * block_size;
            let yo = row_index * block_size;
            draw_cell(framebuffer, xo, yo, block_size, *cell);
        }
    }
}

use std::fs::File;
use std::io::{BufRead, BufReader};

pub type Maze = Vec<Vec<char>>;

pub fn load_maze(filename: &str) -> Maze {
    let file = File::open(filename).expect("No se pudo abrir el archivo del laberinto");
    let reader = BufReader::new(file);

    reader
        .lines()
        .map(|line| line.unwrap().chars().collect::<Vec<char>>())
        .collect()
}

pub fn find_char(maze: &Maze, target: char) -> Option<(usize, usize)> {
    for (j, row) in maze.iter().enumerate() {
        for (i, c) in row.iter().enumerate() {
            if *c == target {
                return Some((i, j));
            }
        }
    }
    None
}

/// Retorna ancho (columnas) y alto (filas) en celdas.
pub fn maze_dims(maze: &Maze) -> (usize, usize) {
    let h = maze.len();
    let w = maze.first().map(|r| r.len()).unwrap_or(0);
    (w, h)
}

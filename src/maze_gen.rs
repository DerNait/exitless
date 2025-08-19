use rand::{seq::SliceRandom, Rng};

/// Genera un laberinto en formato de texto como el del script Python.
/// Retorna String; puedes guardarlo en assets/maze.txt si quieres.
pub fn make_maze_text(w: usize, h: usize) -> String {
    // vis, ver, hor como en el Python
    let mut vis = vec![vec![0; w + 1]; h + 1];
    for y in 0..h { vis[y][w] = 1; }
    for x in 0..=w { vis[h][x] = 1; }

    let mut ver: Vec<Vec<String>> = vec![vec!["|  ".to_string(); w]; h];
    for y in 0..h { ver[y].push("|".to_string()); }
    ver.push(vec![]);

    let mut hor: Vec<Vec<String>> = vec![vec!["+--".to_string(); w]; h+1];
    for y in 0..=h { hor[y].push("+".to_string()); }

    fn walk(x: usize, y: usize, vis: &mut Vec<Vec<i32>>, ver: &mut Vec<Vec<String>>, hor: &mut Vec<Vec<String>>, w: usize, h: usize) {
        vis[y][x] = 1;
        let mut d = vec![
            (x as isize - 1, y as isize),
            (x as isize, y as isize + 1),
            (x as isize + 1, y as isize),
            (x as isize, y as isize - 1),
        ];
        let mut rng = rand::thread_rng();
        d.shuffle(&mut rng);

        for (xx, yy) in d {
            let xxu = xx as usize;
            let yyu = yy as usize;
            if vis[yyu][xxu] != 0 { continue; }
            if xxu == x { hor[y.max(yyu)][x] = "+  ".to_string(); }
            if yyu == y { ver[y][x.max(xxu)] = "   ".to_string(); }
            walk(xxu, yyu, vis, ver, hor, w, h);
        }
    }

    let mut rng = rand::thread_rng();
    walk(rng.gen_range(0..w), rng.gen_range(0..h), &mut vis, &mut ver, &mut hor, w, h);

    // construir texto
    let mut s = String::new();
    for (a, b) in hor.iter().zip(ver.iter()) {
        for part in a { s.push_str(part); }
        s.push('\n');
        for part in b { s.push_str(part); }
        s.push('\n');
    }

    // colocar p y g
    let mut chars: Vec<char> = s.chars().collect();
    let idx_p = w * 3 + 3;
    if idx_p < chars.len() { chars[idx_p] = 'p'; }
    let idx_g = chars.len() as isize - ((w * 3 + 3) as isize) - 3;
    if idx_g >= 0 { chars[idx_g as usize] = 'g'; }

    chars.into_iter().collect()
}

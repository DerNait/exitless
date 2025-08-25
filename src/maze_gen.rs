use rand::{seq::SliceRandom, Rng, SeedableRng, rngs::StdRng};
use std::cmp::Ordering;

/// Config de generación (valores por defecto razonables para “donas” y variedad)
#[derive(Clone, Copy)]
pub struct MazeGenConfig {
    /// Factor de “loops”: proporción aproximada de paredes extra que se derriban tras el backtracker (0..1)
    pub loop_factor: f32,
    /// Número de “salas/donas”: abre rectángulos huecos para generar zonas rodeables
    pub donuts: usize,
    /// Probabilidad de colocar pared especial en el borde superior/derecho (0..1, pequeña)
    pub special_border_prob: f32,
    /// Cantidad base de llaves por tipo (escala con el área)
    pub keys_per_type_base: usize,
    /// Cantidad base de puertas por tipo (se colocan en pasillos, bloqueando)
    pub doors_per_type_base: usize,
    /// Usar semilla fija (opcional)
    pub seed: Option<u64>,
}

impl Default for MazeGenConfig {
    fn default() -> Self {
        Self {
            loop_factor: 0.15,         // abre ~15% aristas extra para loops
            donuts: 6,                 // 6 “donas” por mapa grande
            special_border_prob: 0.04, // 4% en paredes top/right
            keys_per_type_base: 4,     // se escala con el tamaño
            doors_per_type_base: 3,    // se escala con el tamaño
            seed: None,
        }
    }
}

/// Genera un laberinto ASCII con tus símbolos de juego.
/// - `w`,`h` son dimensiones **en celdas “habitables”** (no en píxeles).
/// - El ASCII final usa `+ - |` para muros, `' '` para pasillos,
///   y luego se “pintan” p/e/G/puertas/llaves.
/// Nota: el *formato* resultante es el mismo estilo de tu generador original (grid de caracteres).
pub fn make_maze_text_advanced(w: usize, h: usize, cfg: MazeGenConfig) -> String {
    assert!(w >= 4 && h >= 4, "Usa al menos 4x4 para que quepan ‘donas’ y contenido");
    let mut rng: StdRng = if let Some(seed) = cfg.seed {
        StdRng::seed_from_u64(seed)
    } else {
        // rand 0.9: thread_rng() → rng(), y from_rng ahora pide &mut y NO devuelve Result
        StdRng::from_rng(&mut rand::rng())
    };



    // ---------------------------
    // 1) Backtracker “perfect maze”
    // ---------------------------
    // Matrices auxiliares estilo generador original:
    // vis para celdas; ver/hor para muros verticales/horizontales.
    let mut vis = vec![vec![0; w + 1]; h + 1];
    for y in 0..h { vis[y][w] = 1; }
    for x in 0..=w { vis[h][x] = 1; }

    let mut ver: Vec<Vec<String>> = vec![vec!["|  ".to_string(); w]; h];
    for y in 0..h { ver[y].push("|".to_string()); }
    ver.push(vec![]);

    let mut hor: Vec<Vec<String>> = vec![vec!["+--".to_string(); w]; h + 1];
    for y in 0..=h { hor[y].push("+".to_string()); }

    fn neighbors(x: usize, y: usize, w: usize, h: usize) -> [(isize, isize); 4] {
        [
            (x as isize - 1, y as isize),
            (x as isize, y as isize + 1),
            (x as isize + 1, y as isize),
            (x as isize, y as isize - 1),
        ]
    }

    fn walk(
        x: usize, y: usize,
        vis: &mut [Vec<i32>],
        ver: &mut [Vec<String>],
        hor: &mut [Vec<String>],
        w: usize, h: usize,
        rng: &mut impl Rng,
    ) {
        vis[y][x] = 1;
        let mut d = neighbors(x, y, w, h);
        d.shuffle(rng);

        for (xx, yy) in d {
            // ⬇️ Guardas de límites para evitar -1 → usize::MAX
            if xx < 0 || yy < 0 { continue; }
            let xxu = xx as usize;
            let yyu = yy as usize;
            if xxu >= w || yyu >= h { continue; }

            if vis[yyu][xxu] != 0 { continue; }
            if xxu == x { hor[y.max(yyu)][x] = "+  ".to_string(); }     // abre pared horizontal
            if yyu == y { ver[y][x.max(xxu)] = "   ".to_string(); }     // abre pared vertical
            walk(xxu, yyu, vis, ver, hor, w, h, rng);
        }
    }

    let sx = rng.gen_range(0..w);
    let sy = rng.gen_range(0..h);
    walk(sx, sy, &mut vis, &mut ver, &mut hor, w, h, &mut rng);

    // ---------------------------
    // 2) Extra loops (derribar paredes adicionales)
    // ---------------------------
    let extra_edges = ((w * h) as f32 * cfg.loop_factor).round() as usize;
    for _ in 0..extra_edges {
        if rng.gen_bool(0.5) {
            // intenta abrir una pared vertical interna
            let y = rng.gen_range(0..h);
            let x = rng.gen_range(1..w); // entre celdas
            ver[y][x] = "   ".to_string();
        } else {
            // intenta abrir una pared horizontal interna
            let y = rng.gen_range(1..h);
            let x = rng.gen_range(0..w);
            hor[y][x] = "+  ".to_string();
        }
    }

    // ---------------------------
    // 3) “Donas” (áreas huecas rodeables)
    // ---------------------------
    // Creamos habitaciones huecas 3x3/4x4/5x5 aprox. (en celdas), abriendo muros interiores.
    let donut_count = cfg.donuts.min((w * h) / 8);
    for _ in 0..donut_count {
        let dw = rng.gen_range(3..=5).min(w.saturating_sub(2));
        let dh = rng.gen_range(3..=5).min(h.saturating_sub(2));
        if dw < 3 || dh < 3 { continue; }
        let ox = rng.gen_range(1..=w - dw);
        let oy = rng.gen_range(1..=h - dh);

        // Abrir contorno y parte interior dejando un “anillo”
        // Abrimos horizontalmente las líneas internas
        for yy in oy..(oy + dh) {
            for xx in ox..(ox + dw) {
                if yy > oy && yy < oy + dh - 1 {
                    // interior -> abre muros horizontales entre estas celdas
                    if xx < ox + dw - 1 {
                        // eliminar pared vertical entre (xx,yy) y (xx+1,yy)
                        ver[yy][xx + 1] = "   ".to_string();
                    }
                }
            }
            if yy < oy + dh {
                for xx in ox..(ox + dw) {
                    // abrir muros horizontales entre filas
                    if yy < oy + dh - 1 {
                        hor[yy + 1][xx] = "+  ".to_string();
                    }
                }
            }
        }
    }

    // ---------------------------
    // 4) Reconstruir texto base (sin entidades)
    // ---------------------------
    let mut s = String::new();
    for (a, b) in hor.iter().zip(ver.iter()) {
        for part in a { s.push_str(part); }
        s.push('\n');
        for part in b { s.push_str(part); }
        s.push('\n');
    }

    // Pasar a grid de chars
    let mut grid: Vec<Vec<char>> = s.lines().map(|line| line.chars().collect()).collect();

    // ️⬇️ Normalización robusta: quitar filas vacías y padear al ancho máximo
    grid.retain(|row| !row.is_empty());
    if grid.is_empty() {
        return s; // no hay contenido útil; evita panic
    }
    let max_w = grid.iter().map(|r| r.len()).max().unwrap();
    for row in grid.iter_mut() {
        if row.len() < max_w {
            row.resize(max_w, ' '); // pad con espacio (también caminable)
        }
    }

    let H = grid.len();
    let W = max_w;

    // Helper: posiciones “caminables”: espacio
    let mut floors: Vec<(usize, usize)> = Vec::new();
    for j in 0..H {
        for i in 0..W {
            if grid[j][i] == ' ' { floors.push((i, j)); }
        }
    }
    if floors.is_empty() {
        // No hay pisos; devuelve el ASCII base (sin entidades) en vez de hacer panic
        return s;
    }

    // 5) Colocar jugador (p), salida (G) y enemigo (e), de forma segura
    // Jugador cerca de la esquina superior-izquierda (mínima |i|+|j|)
    let (px, py) = floors
        .iter()
        .min_by_key(|(ix, iy)| (*ix + *iy))
        .copied()
        .unwrap_or((1.min(W-1), 1.min(H-1))); // fallback seguro si algo raro pasa
    grid[py][px] = 'p';

    // Salida G: la más alejada del jugador (euclidiana^2), con fallback seguro
    let far_g = floors
        .iter()
        .max_by_key(|(ix, iy)| {
            let dx = *ix as isize - px as isize;
            let dy = *iy as isize - py as isize;
            (dx*dx + dy*dy) as i64
        })
        .copied()
        .unwrap_or((W.saturating_sub(2), H.saturating_sub(2)));
    grid[far_g.1.min(H-1)][far_g.0.min(W-1)] = 'G';

    // Enemigo e: el más alejado de p que no sea p ni G
    let mut floors2: Vec<(usize, usize)> = floors
        .into_iter()
        .filter(|&(i, j)| (i, j) != (px, py) && (i, j) != far_g)
        .collect();
    let far_e = floors2
        .iter()
        .max_by_key(|(ix, iy)| {
            let dx = *ix as isize - px as isize;
            let dy = *iy as isize - py as isize;
            (dx*dx + dy*dy) as i64
        })
        .copied()
        .unwrap_or((W/2, H/2));
    grid[far_e.1.min(H-1)][far_e.0.min(W-1)] = 'e';


    // ---------------------------
    // 6) Paredes especiales en bordes (arriba & derecha)
    // ---------------------------
    for x in 0..W {
        if grid[0][x] == '-' && rng.gen_bool(cfg.special_border_prob as f64) {
            grid[0][x] = pick(&mut rng, &['#', '!']);
        }
    }
    for y in 0..H {
        if grid[y][W - 1] == '|' && rng.gen_bool(cfg.special_border_prob as f64) {
            grid[y][W - 1] = '@';
        }
    }

    // ---------------------------
    // 7) Llaves y Puertas (múltiples por tipo)
    // ---------------------------
    // Escalado por tamaño del mapa ASCII (no por w,h de celdas)
    let area = (W * H).max(1) as f32;
    let scale = (area / 12_000.0).clamp(0.6, 2.2); // ajusta densidad para mapas grandes
    let keys_per_type  = ((cfg.keys_per_type_base as f32)  * scale).round() as usize;
    let doors_per_type = ((cfg.doors_per_type_base as f32) * scale).round() as usize;

    place_multiple(&mut grid, '1', keys_per_type, &mut rng); // amarilla
    place_multiple(&mut grid, '2', keys_per_type, &mut rng); // azul
    place_multiple(&mut grid, '3', keys_per_type, &mut rng); // roja

    // Las puertas son celdas sólidas que se colocan en corredores (reemplazan un ' ')
    // Evitamos colocarlas a 4 celdas de p para no bloquear el spawn inmediato.
    place_doors(&mut grid, 'Y', doors_per_type, (px, py), 4, &mut rng);
    place_doors(&mut grid, 'B', doors_per_type, (px, py), 4, &mut rng);
    place_doors(&mut grid, 'R', doors_per_type, (px, py), 4, &mut rng);

    // ---------------------------
    // 8) Reconstruir string final
    // ---------------------------
    let mut out = String::with_capacity(W * H + H);
    for (j, row) in grid.into_iter().enumerate() {
        for ch in row {
            out.push(ch);
        }
        if j + 1 < H { out.push('\n'); }
    }
    out
}

#[inline]
fn dist2(ax: isize, ay: isize, bx: isize, by: isize) -> OrderingWrapper {
    let dx = ax - bx;
    let dy = ay - by;
    OrderingWrapper((dx * dx + dy * dy) as i64)
}

// Pequeño wrapper para poder usar cmp() con euclidiana^2
#[derive(Copy, Clone)]
struct OrderingWrapper(i64);
impl PartialEq for OrderingWrapper {
    fn eq(&self, other: &Self) -> bool { self.0 == other.0 }
}
impl Eq for OrderingWrapper {}
impl PartialOrd for OrderingWrapper {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> { Some(self.cmp(other)) }
}
impl Ord for OrderingWrapper {
    fn cmp(&self, other: &Self) -> Ordering { self.0.cmp(&other.0) }
}

#[inline]
fn pick(rng: &mut impl Rng, arr: &[char; 2]) -> char {
    let i = rng.gen_range(0..2);
    arr[i]
}

fn place_multiple(grid: &mut [Vec<char>], ch: char, count: usize, rng: &mut impl Rng) {
    let H = grid.len();
    let W = grid[0].len();
    let mut coords: Vec<(usize, usize)> = Vec::new();
    for j in 0..H {
        for i in 0..W {
            if grid[j][i] == ' ' { coords.push((i, j)); }
        }
    }
    coords.shuffle(rng);
    for (i, (x, y)) in coords.into_iter().enumerate() {
        if i >= count { break; }
        if grid[y][x] == ' ' {
            grid[y][x] = ch;
        }
    }
}

/// Puertas sólidas dentro de pasillos; tratamos de no bloquear el spawn inmediato del jugador.
/// Distancia mínima en Manhattan desde `avoid` (usualmente p).
fn place_doors(
    grid: &mut [Vec<char>],
    door_ch: char,
    count: usize,
    avoid: (usize, usize),
    min_manhattan: usize,
    rng: &mut impl Rng
) {
    let H = grid.len();
    let W = grid[0].len();

    let mut candidates: Vec<(usize, usize)> = Vec::new();
    for j in 0..H {
        for i in 0..W {
            if grid[j][i] != ' ' { continue; }
            // heurística: lugar “pasillo” si tiene exactamente 2 vecinos sólidos opuestos o 2 libres alineados
            let mut free = 0;
            let mut solid = 0;

            let nb = |x: isize, y: isize| -> char {
                if x < 0 || y < 0 || y as usize >= H || x as usize >= W { return '#'; }
                grid[y as usize][x as usize]
            };

            let left  = nb(i as isize - 1, j as isize);
            let right = nb(i as isize + 1, j as isize);
            let up    = nb(i as isize, j as isize - 1);
            let down  = nb(i as isize, j as isize + 1);

            let is_solid = |c: char| matches!(c, '+' | '-' | '|' | '#' | '@' | '!' );
            let is_free  = |c: char| c == ' ' || c == '1' || c == '2' || c == '3';

            // Cuenta simples
            for c in [left, right, up, down] {
                if is_solid(c) { solid += 1; }
                if is_free(c)  { free += 1;  }
            }

            // Debe estar algo “canalizado”
            if !((is_free(left) && is_free(right)) ^ (is_free(up) && is_free(down))) {
                continue;
            }

            // distancia de seguridad al spawn del jugador
            let md = i.abs_diff(avoid.0) + j.abs_diff(avoid.1);
            if md < min_manhattan { continue; }

            // No colocar sobre G, p, e
            if matches!(nb(i as isize, j as isize), 'p' | 'e' | 'G') { continue; }

            candidates.push((i, j));
        }
    }

    candidates.shuffle(rng);
    for (k, (x, y)) in candidates.into_iter().enumerate() {
        if k >= count { break; }
        if grid[y][x] == ' ' {
            grid[y][x] = door_ch;
        }
    }
}

/// Versión compatible con tu firma anterior (mantiene comportamiento “sano” por defecto)
pub fn make_maze_text(w: usize, h: usize) -> String {
    make_maze_text_advanced(w, h, MazeGenConfig::default())
}

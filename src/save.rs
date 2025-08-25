use std::fs;
use std::path::Path;

#[derive(Clone, Debug)]
pub struct Progress {
    pub unlocked: [bool; 3], // L1 siempre true; L2/L3 dependen
    pub last_level: u8,      // 0..2
}

impl Default for Progress {
    fn default() -> Self {
        Self { unlocked: [true, false, false], last_level: 0 }
    }
}

const SAVE_PATH: &str = "assets/save_progress.txt";

pub fn load_progress() -> Progress {
    if let Ok(s) = fs::read_to_string(SAVE_PATH) {
        let mut p = Progress::default();
        for line in s.lines() {
            let t = line.trim();
            if let Some(v) = t.strip_prefix("unlocked=") {
                let parts: Vec<_> = v.split(',').collect();
                if parts.len() == 3 {
                    p.unlocked[0] = parts[0] == "1";
                    p.unlocked[1] = parts[1] == "1";
                    p.unlocked[2] = parts[2] == "1";
                }
            } else if let Some(v) = t.strip_prefix("last_level=") {
                if let Ok(n) = v.parse::<u8>() { p.last_level = n.min(2); }
            }
        }
        return p;
    }
    Progress::default()
}

pub fn save_progress(p: &Progress) {
    let _ = fs::create_dir_all(Path::new("assets"));
    let s = format!(
        "unlocked={},{},{}\nlast_level={}\n",
        if p.unlocked[0] {1} else {0},
        if p.unlocked[1] {1} else {0},
        if p.unlocked[2] {1} else {0},
        p.last_level
    );
    let _ = fs::write(SAVE_PATH, s);
}

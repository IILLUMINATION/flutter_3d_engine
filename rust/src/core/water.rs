use glam::IVec3;
use std::collections::{HashMap, HashSet};

const MAX_LEVEL: u8 = 8;

pub struct WaterSim {
    pub cells: HashMap<IVec3, u8>,
    pub dirty: bool,
}

impl WaterSim {
    pub fn new() -> Self {
        Self {
            cells: HashMap::new(),
            dirty: true,
        }
    }

    pub fn spawn_block(&mut self, center: glam::Vec3, size: u32) {
        let cx = center.x.round() as i32;
        let cy = center.y.round() as i32;
        let cz = center.z.round() as i32;
        let r = size as i32 / 2;
        for dx in -r..=r {
            for dy in -r..=r {
                for dz in -r..=r {
                    let dist = (dx.abs().max(dy.abs())).max(dz.abs());
                    let level = if dist <= 1 { MAX_LEVEL } else { 4 };
                    let key = IVec3::new(cx + dx, cy + dy, cz + dz);
                    self.cells.entry(key).or_insert(level);
                }
            }
        }
        self.dirty = true;
    }

    fn has(&self, x: i32, y: i32, z: i32) -> u8 {
        self.cells.get(&IVec3::new(x, y, z)).copied().unwrap_or(0)
    }

    fn is_solid(&self, x: i32, y: i32, z: i32, solids: &HashSet<IVec3>) -> bool {
        solids.contains(&IVec3::new(x, y, z))
    }

    #[allow(dead_code)]
    fn set(&mut self, x: i32, y: i32, z: i32, v: u8) {
        if y < -64 { return; }
        if v == 0 {
            self.cells.remove(&IVec3::new(x, y, z));
        } else {
            self.cells.insert(IVec3::new(x, y, z), v);
        }
        self.dirty = true;
    }

    pub fn tick(&mut self, solids: &HashSet<IVec3>) {
        let mut changes: Vec<(IVec3, u8)> = Vec::new();

        let keys: Vec<IVec3> = self.cells.keys().copied().collect();
        for key in &keys {
            let level = self.cells.get(key).copied().unwrap_or(0);
            if level == 0 { continue; }
            let x = key.x; let y = key.y; let z = key.z;

            let below = self.has(x, y - 1, z);
            let can_fall = below < MAX_LEVEL * 2 && !self.is_solid(x, y - 1, z, solids);

            if can_fall {
                let space = (MAX_LEVEL * 2 - below).min(level);
                let flow = (space / 2).max(1).min(MAX_LEVEL);
                changes.push((*key, level - flow));
                changes.push((IVec3::new(x, y - 1, z), below + flow));
                continue;
            }

            let spread_level = level;
            let neighbors: [(i32, i32); 4] = [(1, 0), (-1, 0), (0, 1), (0, -1)];
            for (dx, dz) in neighbors {
                let nx = x + dx; let nz = z + dz;
                let n_level = self.has(nx, y, nz);
                let n_below = self.has(nx, y - 1, nz);
                let can_flow = n_below < MAX_LEVEL * 2 && !self.is_solid(nx, y - 1, nz, solids);
                if spread_level > n_level && can_flow {
                    let diff = spread_level - n_level;
                    let transfer = ((diff as f32) * 0.5).ceil() as u8;
                    let transfer = transfer.min(level).min(MAX_LEVEL * 2 - n_level);
                    if transfer > 0 {
                        changes.push((*key, level - transfer));
                        changes.push((IVec3::new(nx, y, nz), n_level + transfer));
                    }
                }
            }
        }

        for (k, v) in changes {
            if v == 0 {
                self.cells.remove(&k);
            } else {
                self.cells.insert(k, v.clamp(1, MAX_LEVEL * 2));
            }
        }
    }

    pub fn count(&self) -> usize { self.cells.len() }
}

impl Default for WaterSim {
    fn default() -> Self { Self::new() }
}

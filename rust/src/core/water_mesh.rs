use crate::core::water::WaterSim;
use glam::IVec3;
use std::collections::HashSet;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct WaterVertex {
    pub position: [f32; 3],
    pub uv: [f32; 2],
    pub ao: f32,
}

pub struct WaterMesh {
    pub vertices: Vec<WaterVertex>,
    pub indices: Vec<u32>,
}

impl WaterMesh {
    pub fn new() -> Self {
        Self {
            vertices: Vec::with_capacity(65536),
            indices: Vec::with_capacity(65536),
        }
    }

    pub fn rebuild(
        &mut self,
        water: &WaterSim,
        solids: &HashSet<IVec3>,
        player_pos: glam::Vec3,
    ) {
        self.vertices.clear();
        self.indices.clear();

        let radius = 48i32;
        let cx = player_pos.x.round() as i32;
        let cz = player_pos.z.round() as i32;

        for (&key, &level) in &water.cells {
            if level == 0 { continue; }
            let x = key.x; let y = key.y; let z = key.z;
            if (x - cx).abs() > radius || (z - cz).abs() > radius { continue; }

            if solids.contains(&IVec3::new(x, y, z)) { continue; }
            if solids.contains(&IVec3::new(x, y - 1, z)) { continue; }

            let above_key = IVec3::new(x, y + 1, z);
            let above_water = water.cells.get(&above_key).copied().unwrap_or(0);
            let above_solid = solids.contains(&above_key);
            if above_water >= 8 || above_solid { continue; }

            let height = (level as f32) / 8.0;
            let h = height - 0.49;
            let fx = x as f32; let fy = y as f32; let fz = z as f32;
            let hsize = 0.5;
            let base_v = self.vertices.len() as u32;

            let ao_tl = Self::corner_ao(water, solids, x, y, z, -1, 1, -1, level);
            let ao_tr = Self::corner_ao(water, solids, x, y, z, 1, 1, -1, level);
            let ao_bl = Self::corner_ao(water, solids, x, y, z, -1, 1, 1, level);
            let ao_br = Self::corner_ao(water, solids, x, y, z, 1, 1, 1, level);

            self.vertices.push(WaterVertex {
                position: [fx - hsize, fy + h, fz - hsize],
                uv: [0.0, 0.0],
                ao: ao_tl,
            });
            self.vertices.push(WaterVertex {
                position: [fx + hsize, fy + h, fz - hsize],
                uv: [1.0, 0.0],
                ao: ao_tr,
            });
            self.vertices.push(WaterVertex {
                position: [fx - hsize, fy + h, fz + hsize],
                uv: [0.0, 1.0],
                ao: ao_bl,
            });
            self.vertices.push(WaterVertex {
                position: [fx + hsize, fy + h, fz + hsize],
                uv: [1.0, 1.0],
                ao: ao_br,
            });

            self.indices.push(base_v);
            self.indices.push(base_v + 2);
            self.indices.push(base_v + 1);
            self.indices.push(base_v + 1);
            self.indices.push(base_v + 2);
            self.indices.push(base_v + 3);
        }
    }

    fn corner_ao(
        water: &WaterSim,
        solids: &HashSet<IVec3>,
        x: i32, y: i32, z: i32,
        sx: i32, sz: i32,
        side: i32,
        level: u8,
    ) -> f32 {
        let s1 = solids.contains(&IVec3::new(x + sx, y + side, z));
        let s2 = solids.contains(&IVec3::new(x, y + side, z + sz));
        let s3 = solids.contains(&IVec3::new(x + sx, y + side, z + sz));
        let w1 = water.cells.get(&IVec3::new(x + sx, y, z)).copied().unwrap_or(0);
        let w2 = water.cells.get(&IVec3::new(x, y, z + sz)).copied().unwrap_or(0);
        let mut occ = 0u32;
        if s1 { occ += 1; }
        if s2 { occ += 1; }
        if s3 { occ += 1; }
        if w1 < level { occ += 1; }
        if w2 < level { occ += 1; }
        let ao: f32 = match occ {
            0 => 1.0,
            1 => 0.7,
            2 => 0.5,
            _ => 0.3,
        };
        ao
    }

    pub fn is_empty(&self) -> bool { self.indices.is_empty() }
}

impl Default for WaterMesh {
    fn default() -> Self { Self::new() }
}

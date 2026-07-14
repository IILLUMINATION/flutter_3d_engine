use glam::Vec3;
use std::collections::HashSet;

pub const PARTICLE_RADIUS: f32 = 0.12;
pub const SMOOTHING_RADIUS: f32 = PARTICLE_RADIUS * 4.0;
pub const REST_DENSITY: f32 = 1000.0;
pub const PARTICLE_MASS: f32 = 0.02;
pub const GAS_CONSTANT: f32 = 800.0;
pub const VISCOSITY: f32 = 0.15;
pub const GRAVITY: f32 = -9.81;
pub const BOUNDARY_DAMPING: f32 = 0.4;
pub const MAX_PARTICLES: usize = 4096;

#[derive(Debug, Clone, Copy)]
pub struct FluidParticle {
    pub position: Vec3,
    pub velocity: Vec3,
    pub density:  f32,
    pub pressure: f32,
}

pub struct FluidSystem {
    pub particles: Vec<FluidParticle>,
}

impl FluidSystem {
    pub fn new() -> Self {
        Self { particles: Vec::with_capacity(MAX_PARTICLES) }
    }

    pub fn count(&self) -> usize { self.particles.len() }

    pub fn spawn_block(&mut self, origin: Vec3, nx: u32, ny: u32, nz: u32) {
        let spacing = PARTICLE_RADIUS * 2.2;
        for i in 0..nx {
            for j in 0..ny {
                for k in 0..nz {
                    if self.particles.len() >= MAX_PARTICLES { return; }
                    let pos = origin + Vec3::new(i as f32, j as f32, k as f32) * spacing;
                    self.particles.push(FluidParticle {
                        position: pos,
                        velocity: Vec3::ZERO,
                        density: 0.0,
                        pressure: 0.0,
                    });
                }
            }
        }
    }

    fn poly6_kernel(r: f32, h: f32) -> f32 {
        if r >= h { return 0.0; }
        let h2 = h * h;
        let diff = h2 - r * r;
        315.0 / (64.0 * std::f32::consts::PI * h.powi(9)) * diff * diff * diff
    }

    fn spiky_grad_kernel(r: f32, h: f32) -> f32 {
        if r >= h || r <= 0.0001 { return 0.0; }
        -45.0 / (std::f32::consts::PI * h.powi(6)) * (h - r) * (h - r) / r
    }

    fn viscosity_laplacian_kernel(r: f32, h: f32) -> f32 {
        if r >= h { return 0.0; }
        45.0 / (std::f32::consts::PI * h.powi(6)) * (h - r)
    }

    pub fn step(&mut self, dt: f32, solid_aabbs: &[(Vec3, Vec3)]) {
        let n = self.particles.len();
        if n == 0 { return; }

        let h = SMOOTHING_RADIUS;
        let grid_size = h;
        let grid_min = Vec3::new(-200.0, -200.0, -200.0);

        let positions: Vec<Vec3> = self.particles.iter().map(|p| p.position).collect();

        let mut grid: Vec<Vec<usize>> = vec![Vec::new(); 64 * 64 * 64];

        for (i, &pos) in positions.iter().enumerate() {
            let ix = ((pos.x - grid_min.x) / grid_size) as usize;
            let iy = ((pos.y - grid_min.y) / grid_size) as usize;
            let iz = ((pos.z - grid_min.z) / grid_size) as usize;
            if ix < 64 && iy < 64 && iz < 64 {
                let ci = ix + iy * 64 + iz * 64 * 64;
                if ci < grid.len() { grid[ci].push(i); }
            }
        }

        for i in 0..n {
            let mut density = 0.0f32;
            let ix = ((positions[i].x - grid_min.x) / grid_size) as usize;
            let iy = ((positions[i].y - grid_min.y) / grid_size) as usize;
            let iz = ((positions[i].z - grid_min.z) / grid_size) as usize;
            if ix >= 64 || iy >= 64 || iz >= 64 { continue; }

            for dx in 0u32..=2u32 {
                for dy in 0u32..=2u32 {
                    for dz in 0u32..=2u32 {
                        let nx = ix.wrapping_add(dx as usize - 1);
                        let ny = iy.wrapping_add(dy as usize - 1);
                        let nz = iz.wrapping_add(dz as usize - 1);
                        if nx >= 64 || ny >= 64 || nz >= 64 { continue; }
                        let ci = nx + ny * 64 + nz * 64 * 64;
                        if ci >= grid.len() { continue; }
                        for &j in &grid[ci] {
                            let r = (positions[i] - positions[j]).length();
                            density += PARTICLE_MASS * Self::poly6_kernel(r, h);
                        }
                    }
                }
            }
            self.particles[i].density = density.max(0.01);
            self.particles[i].pressure = GAS_CONSTANT * (density - REST_DENSITY).max(0.0);
        }

        let pressures: Vec<f32> = self.particles.iter().map(|p| p.pressure).collect();
        let densities: Vec<f32> = self.particles.iter().map(|p| p.density).collect();

        let mut forces = vec![Vec3::ZERO; n];
        for i in 0..n {
            let ix = ((positions[i].x - grid_min.x) / grid_size) as usize;
            let iy = ((positions[i].y - grid_min.y) / grid_size) as usize;
            let iz = ((positions[i].z - grid_min.z) / grid_size) as usize;
            if ix >= 64 || iy >= 64 || iz >= 64 { continue; }

            for dx in 0u32..=2u32 {
                for dy in 0u32..=2u32 {
                    for dz in 0u32..=2u32 {
                        let nx = ix.wrapping_add(dx as usize - 1);
                        let ny = iy.wrapping_add(dy as usize - 1);
                        let nz = iz.wrapping_add(dz as usize - 1);
                        if nx >= 64 || ny >= 64 || nz >= 64 { continue; }
                        let ci = nx + ny * 64 + nz * 64 * 64;
                        if ci >= grid.len() { continue; }
                        for &j in &grid[ci] {
                            if i == j { continue; }
                            let dir = positions[i] - positions[j];
                            let r = dir.length();
                            if r < 0.0001 || r >= h { continue; }
                            let dir_norm = dir / r;

                            let avg_p = (pressures[i] + pressures[j]) * 0.5;
                            let rho_j = densities[j];
                            forces[i] += dir_norm * PARTICLE_MASS * avg_p / rho_j * Self::spiky_grad_kernel(r, h);

                            let vel_diff = self.particles[j].velocity - self.particles[i].velocity;
                            forces[i] += vel_diff * VISCOSITY * PARTICLE_MASS / rho_j * Self::viscosity_laplacian_kernel(r, h);
                        }
                    }
                }
            }
            forces[i].y += GRAVITY * PARTICLE_MASS;
        }

        for i in 0..n {
            let dens = self.particles[i].density;
            if dens < 0.01 { continue; }
            let f = forces[i];
            let vel = self.particles[i].velocity;
            let new_vel = vel + f / dens * dt;
            let new_pos = self.particles[i].position + new_vel * dt;

            let mut final_pos = new_pos;
            let mut final_vel = new_vel;
            for &(bmin, bmax) in solid_aabbs {
                let r = PARTICLE_RADIUS;
                if final_pos.x - r < bmin.x { final_pos.x = bmin.x + r; final_vel.x *= -BOUNDARY_DAMPING; }
                if final_pos.x + r > bmax.x { final_pos.x = bmax.x - r; final_vel.x *= -BOUNDARY_DAMPING; }
                if final_pos.y - r < bmin.y { final_pos.y = bmin.y + r; final_vel.y *= -BOUNDARY_DAMPING; }
                if final_pos.y + r > bmax.y { final_pos.y = bmax.y - r; final_vel.y *= -BOUNDARY_DAMPING; }
                if final_pos.z - r < bmin.z { final_pos.z = bmin.z + r; final_vel.z *= -BOUNDARY_DAMPING; }
                if final_pos.z + r > bmax.z { final_pos.z = bmax.z - r; final_vel.z *= -BOUNDARY_DAMPING; }
            }

            let max_vel = 30.0;
            if final_vel.length() > max_vel {
                final_vel = final_vel.normalize_or_zero() * max_vel;
            }

            self.particles[i].position = final_pos;
            self.particles[i].velocity = final_vel;
        }
    }
}

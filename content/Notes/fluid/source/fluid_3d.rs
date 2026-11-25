use macroquad::prelude::*;

const N: usize = 32;
const SIZE: usize = (N + 2) * (N + 2) * (N + 2);
const NUM_PARTICLES: usize = 6000;
const BOX_SIZE: f32 = 1.6;

#[inline]
fn ix(x: usize, y: usize, z: usize) -> usize {
    x + (N + 2) * (y + (N + 2) * z)
}

// The caller keeps pos inside the boundary cells.
#[inline]
fn sample_field(field: &[f32], pos: Vec3) -> f32 {
    let (i, j, k) = (pos.x as usize, pos.y as usize, pos.z as usize);
    let (x, y, z) = (pos.x - i as f32, pos.y - j as f32, pos.z - k as f32);
    (1.0 - x)
        * ((1.0 - y) * ((1.0 - z) * field[ix(i, j, k)] + z * field[ix(i, j, k + 1)])
            + y * ((1.0 - z) * field[ix(i, j + 1, k)] + z * field[ix(i, j + 1, k + 1)]))
        + x * ((1.0 - y) * ((1.0 - z) * field[ix(i + 1, j, k)] + z * field[ix(i + 1, j, k + 1)])
            + y * ((1.0 - z) * field[ix(i + 1, j + 1, k)] + z * field[ix(i + 1, j + 1, k + 1)]))
}

struct Fluid3D {
    dt: f32,
    u: Vec<f32>,
    v: Vec<f32>,
    w: Vec<f32>,
    u_prev: Vec<f32>,
    v_prev: Vec<f32>,
    w_prev: Vec<f32>,
    curl_x: Vec<f32>,
    curl_y: Vec<f32>,
    curl_z: Vec<f32>,
    curl_mag: Vec<f32>,
}

impl Fluid3D {
    fn new(dt: f32) -> Self {
        Self {
            dt,
            u: vec![0.0; SIZE],
            v: vec![0.0; SIZE],
            w: vec![0.0; SIZE],
            u_prev: vec![0.0; SIZE],
            v_prev: vec![0.0; SIZE],
            w_prev: vec![0.0; SIZE],
            curl_x: vec![0.0; SIZE],
            curl_y: vec![0.0; SIZE],
            curl_z: vec![0.0; SIZE],
            curl_mag: vec![0.0; SIZE],
        }
    }

    fn set_bnd(b: i32, x: &mut [f32]) {
        let tangential = if b == 0 { 1.0 } else { 0.7 };
        for j in 1..=N {
            for i in 1..=N {
                x[ix(0, i, j)] = if b == 1 {
                    -x[ix(1, i, j)]
                } else {
                    tangential * x[ix(1, i, j)]
                };
                x[ix(N + 1, i, j)] = if b == 1 {
                    -x[ix(N, i, j)]
                } else {
                    tangential * x[ix(N, i, j)]
                };
                x[ix(i, 0, j)] = if b == 2 {
                    -x[ix(i, 1, j)]
                } else {
                    tangential * x[ix(i, 1, j)]
                };
                x[ix(i, N + 1, j)] = if b == 2 {
                    -x[ix(i, N, j)]
                } else {
                    tangential * x[ix(i, N, j)]
                };
                x[ix(i, j, 0)] = if b == 3 {
                    -x[ix(i, j, 1)]
                } else {
                    tangential * x[ix(i, j, 1)]
                };
                x[ix(i, j, N + 1)] = if b == 3 {
                    -x[ix(i, j, N)]
                } else {
                    tangential * x[ix(i, j, N)]
                };
            }
        }
    }

    fn lin_solve(b: i32, x: &mut [f32], x0: &[f32], a: f32, c: f32) {
        let inv_c = 1.0 / c;
        for _ in 0..6 {
            for k in 1..=N {
                for j in 1..=N {
                    for i in 1..=N {
                        x[ix(i, j, k)] = (x0[ix(i, j, k)]
                            + a * (x[ix(i - 1, j, k)]
                                + x[ix(i + 1, j, k)]
                                + x[ix(i, j - 1, k)]
                                + x[ix(i, j + 1, k)]
                                + x[ix(i, j, k - 1)]
                                + x[ix(i, j, k + 1)]))
                            * inv_c;
                    }
                }
            }
            Self::set_bnd(b, x);
        }
    }

    fn advect(b: i32, d: &mut [f32], d0: &[f32], u: &[f32], v: &[f32], w: &[f32], dt: f32) {
        let dt0 = dt * N as f32;
        for k in 1..=N {
            for j in 1..=N {
                for i in 1..=N {
                    let x = (i as f32 - dt0 * u[ix(i, j, k)]).clamp(0.5, N as f32 + 0.5);
                    let y = (j as f32 - dt0 * v[ix(i, j, k)]).clamp(0.5, N as f32 + 0.5);
                    let z = (k as f32 - dt0 * w[ix(i, j, k)]).clamp(0.5, N as f32 + 0.5);

                    d[ix(i, j, k)] = sample_field(d0, vec3(x, y, z));
                }
            }
        }
        Self::set_bnd(b, d);
    }

    fn project(&mut self) {
        let h = 1.0 / N as f32;
        let p = &mut self.u_prev;
        let div = &mut self.v_prev;

        for k in 1..=N {
            for j in 1..=N {
                for i in 1..=N {
                    div[ix(i, j, k)] = -0.5
                        * h
                        * (self.u[ix(i + 1, j, k)] - self.u[ix(i - 1, j, k)]
                            + self.v[ix(i, j + 1, k)]
                            - self.v[ix(i, j - 1, k)]
                            + self.w[ix(i, j, k + 1)]
                            - self.w[ix(i, j, k - 1)]);
                    p[ix(i, j, k)] = 0.0;
                }
            }
        }
        Self::set_bnd(0, div);
        Self::set_bnd(0, p);
        Self::lin_solve(0, p, div, 1.0, 6.0);

        for k in 1..=N {
            for j in 1..=N {
                for i in 1..=N {
                    self.u[ix(i, j, k)] -= 0.5 * (p[ix(i + 1, j, k)] - p[ix(i - 1, j, k)]) / h;
                    self.v[ix(i, j, k)] -= 0.5 * (p[ix(i, j + 1, k)] - p[ix(i, j - 1, k)]) / h;
                    self.w[ix(i, j, k)] -= 0.5 * (p[ix(i, j, k + 1)] - p[ix(i, j, k - 1)]) / h;
                }
            }
        }
        Self::set_bnd(1, &mut self.u);
        Self::set_bnd(2, &mut self.v);
        Self::set_bnd(3, &mut self.w);
    }

    // Vorticity confinement reinforces vortices in three dimensions.
    fn apply_vorticity(&mut self, strength: f32) {
        for k in 1..=N {
            for j in 1..=N {
                for i in 1..=N {
                    let idx = ix(i, j, k);
                    let dw_dy = (self.w[ix(i, j + 1, k)] - self.w[ix(i, j - 1, k)]) * 0.5;
                    let dv_dz = (self.v[ix(i, j, k + 1)] - self.v[ix(i, j, k - 1)]) * 0.5;
                    let du_dz = (self.u[ix(i, j, k + 1)] - self.u[ix(i, j, k - 1)]) * 0.5;
                    let dw_dx = (self.w[ix(i + 1, j, k)] - self.w[ix(i - 1, j, k)]) * 0.5;
                    let dv_dx = (self.v[ix(i + 1, j, k)] - self.v[ix(i - 1, j, k)]) * 0.5;
                    let du_dy = (self.u[ix(i, j + 1, k)] - self.u[ix(i, j - 1, k)]) * 0.5;

                    let cx = dw_dy - dv_dz;
                    let cy = du_dz - dw_dx;
                    let cz = dv_dx - du_dy;

                    self.curl_x[idx] = cx;
                    self.curl_y[idx] = cy;
                    self.curl_z[idx] = cz;
                    self.curl_mag[idx] = (cx * cx + cy * cy + cz * cz).sqrt();
                }
            }
        }

        for k in 2..N {
            for j in 2..N {
                for i in 2..N {
                    let idx = ix(i, j, k);
                    let dx =
                        (self.curl_mag[ix(i + 1, j, k)] - self.curl_mag[ix(i - 1, j, k)]) * 0.5;
                    let dy =
                        (self.curl_mag[ix(i, j + 1, k)] - self.curl_mag[ix(i, j - 1, k)]) * 0.5;
                    let dz =
                        (self.curl_mag[ix(i, j, k + 1)] - self.curl_mag[ix(i, j, k - 1)]) * 0.5;
                    let len = (dx * dx + dy * dy + dz * dz).sqrt() + 1e-5;

                    let nx = dx / len;
                    let ny = dy / len;
                    let nz = dz / len;

                    let cx = self.curl_x[idx];
                    let cy = self.curl_y[idx];
                    let cz = self.curl_z[idx];

                    // N × curl
                    self.u[idx] += strength * self.dt * (ny * cz - nz * cy);
                    self.v[idx] += strength * self.dt * (nz * cx - nx * cz);
                    self.w[idx] += strength * self.dt * (nx * cy - ny * cx);
                }
            }
        }
    }

    fn step(&mut self) {
        self.apply_vorticity(14.0);

        std::mem::swap(&mut self.u_prev, &mut self.u);
        std::mem::swap(&mut self.v_prev, &mut self.v);
        std::mem::swap(&mut self.w_prev, &mut self.w);
        Self::advect(
            1,
            &mut self.u,
            &self.u_prev,
            &self.u_prev,
            &self.v_prev,
            &self.w_prev,
            self.dt,
        );
        Self::advect(
            2,
            &mut self.v,
            &self.v_prev,
            &self.u_prev,
            &self.v_prev,
            &self.w_prev,
            self.dt,
        );
        Self::advect(
            3,
            &mut self.w,
            &self.w_prev,
            &self.u_prev,
            &self.v_prev,
            &self.w_prev,
            self.dt,
        );

        self.project();

        // Gradually damp the flow after input stops.
        for i in 0..SIZE {
            self.u[i] *= 0.997;
            self.v[i] *= 0.997;
            self.w[i] *= 0.997;
        }
    }

    // Sample the 3D velocity field using trilinear interpolation.
    fn sample_velocity(&self, pos: Vec3) -> Vec3 {
        let pos = pos.clamp(Vec3::splat(1.0), Vec3::splat(N as f32 - 0.001));
        vec3(
            sample_field(&self.u, pos),
            sample_field(&self.v, pos),
            sample_field(&self.w, pos),
        )
    }

    fn splat(&mut self, grid_pos: Vec3, force: Vec3, radius: i32) {
        let cx = grid_pos.x as i32;
        let cy = grid_pos.y as i32;
        let cz = grid_pos.z as i32;
        let r2 = (radius * radius) as f32;

        for dz in -radius..=radius {
            for dy in -radius..=radius {
                for dx in -radius..=radius {
                    let d2 = (dx * dx + dy * dy + dz * dz) as f32;
                    if d2 <= r2 {
                        let weight = (-d2 / (r2 * 0.45)).exp();
                        let x = (cx + dx).clamp(1, N as i32) as usize;
                        let y = (cy + dy).clamp(1, N as i32) as usize;
                        let z = (cz + dz).clamp(1, N as i32) as usize;
                        let idx = ix(x, y, z);

                        self.u[idx] += force.x * weight;
                        self.v[idx] += force.y * weight;
                        self.w[idx] += force.z * weight;
                    }
                }
            }
        }
    }
}

struct Particle {
    pos: Vec3,
    prev_pos: Vec3,
    base_color: Vec3,
    life: f32,
    max_life: f32,
    active: bool,
}

fn hsv_to_rgb(h: f32, s: f32, v: f32) -> Vec3 {
    let i = (h * 6.0).floor() as i32;
    let f = h * 6.0 - i as f32;
    let p = v * (1.0 - s);
    let q = v * (1.0 - f * s);
    let t = v * (1.0 - (1.0 - f) * s);
    let (r, g, b) = match i.rem_euclid(6) {
        0 => (v, t, p),
        1 => (q, v, p),
        2 => (p, v, t),
        3 => (p, q, v),
        4 => (t, p, v),
        _ => (v, p, q),
    };
    vec3(r, g, b)
}

fn update_particles(particles: &mut [Particle], fluid: &Fluid3D, dt: f32) {
    // Move active particles along the velocity field.
    for p in particles {
        if !p.active {
            continue;
        }
        p.prev_pos = p.pos;
        let vel = fluid.sample_velocity(p.pos);
        p.pos += vel * (N as f32) * dt;
        p.life -= dt;

        // Keep particles inside the walls.
        p.pos = p.pos.clamp(Vec3::splat(1.1), Vec3::splat(N as f32 - 0.1));

        if p.life <= 0.0 {
            p.active = false;
        }
    }
}

fn draw_particles(particles: &[Particle]) {
    // Draw particle trails, brightening fast motion and areas near walls.
    let scale = BOX_SIZE / N as f32;
    let half_n = N as f32 * 0.5;
    for p in particles {
        if !p.active {
            continue;
        }
        let world_pos = (p.pos - Vec3::splat(half_n)) * scale;
        let world_prev = (p.prev_pos - Vec3::splat(half_n)) * scale;

        let life_ratio = (p.life / (p.max_life * 0.3)).clamp(0.0, 1.0);
        let dist_sq = world_pos.distance_squared(world_prev);
        let wall_distance = p.pos.min(Vec3::splat(N as f32) - p.pos).min_element();
        let wall_glow = (1.0 - wall_distance / 4.0).clamp(0.0, 1.0) * 0.25;
        let speed_boost = ((dist_sq * 1200.0).min(0.65) + wall_glow).min(0.8);

        let r = (p.base_color.x + speed_boost).min(1.0);
        let g = (p.base_color.y + speed_boost).min(1.0);
        let b = (p.base_color.z + speed_boost).min(1.0);
        let alpha = life_ratio.powf(0.7) * 0.9;
        let c = Color::new(r, g, b, alpha);

        if dist_sq > 0.000004 {
            draw_line_3d(world_prev, world_pos, c);
        } else {
            draw_line_3d(world_pos, world_pos + vec3(0.003, 0.003, 0.003), c);
        }
    }
}

#[macroquad::main("3D Fluid")]
async fn main() {
    request_new_screen_size(900.0, 900.0);

    let mut fluid = Fluid3D::new(0.016);
    let mut particles: Vec<Particle> = (0..NUM_PARTICLES)
        .map(|_| Particle {
            pos: Vec3::ZERO,
            prev_pos: Vec3::ZERO,
            base_color: Vec3::ONE,
            life: 0.0,
            max_life: 1.0,
            active: false,
        })
        .collect();
    let mut next_particle_idx: usize = 0;

    let mut cam_yaw: f32 = 0.65;
    let mut cam_pitch: f32 = 0.38;
    let mut cam_dist: f32 = 3.1;
    let mut prev_mouse: Option<Vec2> = None;
    let mut prev_world_hit: Option<Vec3> = None;
    let mut hue: f32 = 0.0;

    loop {
        let dt = get_frame_time();
        clear_background(Color::from_rgba(24, 24, 27, 255));

        // Zoom with the mouse wheel.
        let (_, wheel_y) = mouse_wheel();
        if wheel_y.abs() > 0.0 {
            cam_dist = (cam_dist - wheel_y * 0.15).clamp(1.2, 4.5);
        }

        let mouse = vec2(mouse_position().0, mouse_position().1);
        let rotating = is_mouse_button_down(MouseButton::Left)
            && (is_key_down(KeyCode::LeftShift) || is_key_down(KeyCode::RightShift));

        // Rotate the camera with Shift + left drag.
        if rotating && !is_mouse_button_pressed(MouseButton::Left) {
            if let Some(pm) = prev_mouse {
                let delta = mouse - pm;
                cam_yaw -= delta.x * 0.007;
                cam_pitch = (cam_pitch + delta.y * 0.007).clamp(-1.45, 1.45);
            }
        }

        let cam_pos = vec3(
            cam_dist * cam_pitch.cos() * cam_yaw.sin(),
            cam_dist * cam_pitch.sin(),
            cam_dist * cam_pitch.cos() * cam_yaw.cos(),
        );
        let forward = (-cam_pos).normalize();
        let right = forward.cross(vec3(0.0, 1.0, 0.0)).normalize();
        let up = right.cross(forward).normalize();

        // Stir the flow and emit particles with a left drag.
        if is_mouse_button_down(MouseButton::Left) && !rotating {
            let ndc_x = (mouse.x / screen_width()) * 2.0 - 1.0;
            let ndc_y = 1.0 - (mouse.y / screen_height()) * 2.0;
            let aspect = screen_width() / screen_height();
            let fov_scale = (45.0_f32.to_radians() * 0.5).tan();

            let ray_dir =
                (forward + right * (ndc_x * aspect * fov_scale) + up * (ndc_y * fov_scale))
                    .normalize();

            // Intersect the mouse ray with the camera-facing plane through the origin.
            let denom = ray_dir.dot(forward);
            let t = (-cam_pos).dot(forward) / denom;
            let hit_world = (cam_pos + ray_dir * t)
                .clamp(Vec3::splat(-BOX_SIZE * 0.45), Vec3::splat(BOX_SIZE * 0.45));

            if let Some(prev_hit) = prev_world_hit {
                let move_vec = hit_world - prev_hit;
                let speed = move_vec.length();

                if speed > 0.0005 {
                    let grid_pos = (hit_world / BOX_SIZE + 0.5) * N as f32;
                    let force = move_vec * 140.0;
                    fluid.splat(grid_pos, force, 3);

                    hue = (hue + 0.006 + speed * 0.25) % 1.0;

                    // Emit particles according to mouse movement.
                    let spawn_count = (25 + (speed * 900.0) as usize).min(90);
                    for _ in 0..spawn_count {
                        let jitter = vec3(
                            rand::gen_range(-1.3, 1.3),
                            rand::gen_range(-1.3, 1.3),
                            rand::gen_range(-1.3, 1.3),
                        );
                        let p = &mut particles[next_particle_idx];
                        p.pos = (grid_pos + jitter)
                            .clamp(Vec3::splat(1.2), Vec3::splat(N as f32 - 0.2));
                        p.prev_pos = p.pos;
                        p.base_color = hsv_to_rgb(
                            (hue + rand::gen_range(-0.04, 0.04)).rem_euclid(1.0),
                            0.85,
                            1.0,
                        );
                        p.max_life = 5.0;
                        p.life = p.max_life;
                        p.active = true;
                        next_particle_idx = (next_particle_idx + 1) % NUM_PARTICLES;
                    }
                }
            }
            prev_world_hit = Some(hit_world);
        } else {
            prev_world_hit = None;
        }
        prev_mouse = Some(mouse);

        fluid.step();

        update_particles(&mut particles, &fluid, dt);

        set_camera(&Camera3D {
            position: cam_pos,
            target: Vec3::ZERO,
            up: vec3(0.0, 1.0, 0.0),
            ..Default::default()
        });

        // Draw the simulation boundary.
        draw_cube_wires(
            Vec3::ZERO,
            Vec3::splat(BOX_SIZE),
            Color::from_rgba(161, 161, 170, 166),
        );

        draw_particles(&particles);

        set_default_camera();
        next_frame().await
    }
}

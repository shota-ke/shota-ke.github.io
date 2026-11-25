use macroquad::prelude::*;

// Number of interior cells per axis; one extra layer holds boundary values.
const N: usize = 160;
const SIZE: usize = (N + 2) * (N + 2);

struct FluidGrid {
    dt: f32,     // Fixed simulation time step, independent of frame time
    diff: f32,   // Dye diffusion coefficient
    visc: f32,   // Velocity diffusion coefficient (viscosity)
    u: Vec<f32>, // X velocity
    v: Vec<f32>, // Y velocity
    // Reuse these buffers for previous values and projection calculations.
    u_prev: Vec<f32>,
    v_prev: Vec<f32>,
    r: Vec<f32>, // Red dye density
    g: Vec<f32>, // Green dye density
    b: Vec<f32>, // Blue dye density
    r_prev: Vec<f32>,
    g_prev: Vec<f32>,
    b_prev: Vec<f32>,
}

// Map grid coordinates, including boundary cells, to a flat array.
#[inline]
fn ix(x: usize, y: usize) -> usize {
    x + (N + 2) * y
}

impl FluidGrid {
    fn new(dt: f32, diff: f32, visc: f32) -> Self {
        Self {
            dt,
            diff,
            visc,
            u: vec![0.0; SIZE],
            v: vec![0.0; SIZE],
            u_prev: vec![0.0; SIZE],
            v_prev: vec![0.0; SIZE],
            r: vec![0.0; SIZE],
            g: vec![0.0; SIZE],
            b: vec![0.0; SIZE],
            r_prev: vec![0.0; SIZE],
            g_prev: vec![0.0; SIZE],
            b_prev: vec![0.0; SIZE],
        }
    }

    // b=0: pressure or density; b=1: X velocity; b=2: Y velocity. Reflect the normal velocity.
    fn set_bnd(b: i32, x: &mut [f32]) {
        for i in 1..=N {
            x[ix(0, i)] = if b == 1 { -x[ix(1, i)] } else { x[ix(1, i)] };
            x[ix(N + 1, i)] = if b == 1 { -x[ix(N, i)] } else { x[ix(N, i)] };
            x[ix(i, 0)] = if b == 2 { -x[ix(i, 1)] } else { x[ix(i, 1)] };
            x[ix(i, N + 1)] = if b == 2 { -x[ix(i, N)] } else { x[ix(i, N)] };
        }
        x[ix(0, 0)] = 0.5 * (x[ix(1, 0)] + x[ix(0, 1)]);
        x[ix(0, N + 1)] = 0.5 * (x[ix(1, N + 1)] + x[ix(0, N)]);
        x[ix(N + 1, 0)] = 0.5 * (x[ix(N, 0)] + x[ix(N + 1, 1)]);
        x[ix(N + 1, N + 1)] = 0.5 * (x[ix(N, N + 1)] + x[ix(N + 1, N)]);
    }

    // Approximate diffusion and pressure with 10 Gauss-Seidel iterations.
    fn lin_solve(b: i32, x: &mut [f32], x0: &[f32], a: f32, c: f32) {
        let inv_c = 1.0 / c;
        for _ in 0..10 {
            for j in 1..=N {
                for i in 1..=N {
                    x[ix(i, j)] = (x0[ix(i, j)]
                        + a * (x[ix(i - 1, j)]
                            + x[ix(i + 1, j)]
                            + x[ix(i, j - 1)]
                            + x[ix(i, j + 1)]))
                        * inv_c;
                }
            }
            Self::set_bnd(b, x);
        }
    }

    // Diffuse a field across the grid.
    fn diffuse(b: i32, x: &mut [f32], x0: &[f32], diff: f32, dt: f32) {
        let a = dt * diff * (N * N) as f32;
        Self::lin_solve(b, x, x0, a, 1.0 + 4.0 * a);
    }

    // Trace the velocity backward and interpolate the four neighboring cells.
    fn advect(b: i32, d: &mut [f32], d0: &[f32], u: &[f32], v: &[f32], dt: f32) {
        let dt0 = dt * N as f32;
        for j in 1..=N {
            for i in 1..=N {
                let x = (i as f32 - dt0 * u[ix(i, j)]).clamp(0.5, N as f32 + 0.5);
                let y = (j as f32 - dt0 * v[ix(i, j)]).clamp(0.5, N as f32 + 0.5);

                let i0 = x as usize;
                let i1 = i0 + 1;
                let j0 = y as usize;
                let j1 = j0 + 1;

                let s1 = x - i0 as f32;
                let s0 = 1.0 - s1;
                let t1 = y - j0 as f32;
                let t0 = 1.0 - t1;

                d[ix(i, j)] = s0 * (t0 * d0[ix(i0, j0)] + t1 * d0[ix(i0, j1)])
                    + s1 * (t0 * d0[ix(i1, j0)] + t1 * d0[ix(i1, j1)]);
            }
        }
        Self::set_bnd(b, d);
    }

    // Subtract the approximate pressure gradient to reduce velocity divergence.
    fn project(u: &mut [f32], v: &mut [f32], p: &mut [f32], div: &mut [f32]) {
        let h = 1.0 / N as f32;
        for j in 1..=N {
            for i in 1..=N {
                div[ix(i, j)] = -0.5
                    * h
                    * (u[ix(i + 1, j)] - u[ix(i - 1, j)] + v[ix(i, j + 1)] - v[ix(i, j - 1)]);
                p[ix(i, j)] = 0.0;
            }
        }
        Self::set_bnd(0, div);
        Self::set_bnd(0, p);
        Self::lin_solve(0, p, div, 1.0, 4.0);

        for j in 1..=N {
            for i in 1..=N {
                u[ix(i, j)] -= 0.5 * (p[ix(i + 1, j)] - p[ix(i - 1, j)]) / h;
                v[ix(i, j)] -= 0.5 * (p[ix(i, j + 1)] - p[ix(i, j - 1)]) / h;
            }
        }
        Self::set_bnd(1, u);
        Self::set_bnd(2, v);
    }

    // Vorticity confinement restores small vortices lost to numerical diffusion.
    fn apply_vorticity(&mut self, strength: f32) {
        let mut curl = vec![0.0; SIZE];
        for j in 1..=N {
            for i in 1..=N {
                let du_dy = (self.u[ix(i, j + 1)] - self.u[ix(i, j - 1)]) * 0.5;
                let dv_dx = (self.v[ix(i + 1, j)] - self.v[ix(i - 1, j)]) * 0.5;
                curl[ix(i, j)] = dv_dx - du_dy;
            }
        }

        for j in 2..N {
            for i in 2..N {
                let dw_dx = (curl[ix(i + 1, j)].abs() - curl[ix(i - 1, j)].abs()) * 0.5;
                let dw_dy = (curl[ix(i, j + 1)].abs() - curl[ix(i, j - 1)].abs()) * 0.5;
                let len = (dw_dx * dw_dx + dw_dy * dw_dy).sqrt() + 1e-5;
                let nx = dw_dx / len;
                let ny = dw_dy / len;

                let w = curl[ix(i, j)];
                self.u[ix(i, j)] += strength * self.dt * (ny * w);
                self.v[ix(i, j)] -= strength * self.dt * (nx * w);
            }
        }
    }

    // Add velocity and dye around the cursor.
    fn inject(&mut self, gx: i32, gy: i32, dx: f32, dy: f32, cr: f32, cg: f32, cb: f32) {
        // Inject within a three-cell radius.
        let radius = 3;
        for oy in -radius..=radius {
            for ox in -radius..=radius {
                if ox * ox + oy * oy <= radius * radius {
                    let x = (gx + ox).clamp(1, N as i32) as usize;
                    let y = (gy + oy).clamp(1, N as i32) as usize;
                    let idx = ix(x, y);

                    self.u[idx] += dx * 0.35;
                    self.v[idx] += dy * 0.35;
                    self.r[idx] = (self.r[idx] + cr * 0.8).min(2.5);
                    self.g[idx] = (self.g[idx] + cg * 0.8).min(2.5);
                    self.b[idx] = (self.b[idx] + cb * 0.8).min(2.5);
                }
            }
        }
    }

    // Advance the simulation by one fixed time step.
    fn step(&mut self) {
        self.apply_vorticity(18.0);

        // Update velocity: diffuse, project, advect, and project again.
        std::mem::swap(&mut self.u_prev, &mut self.u);
        Self::diffuse(1, &mut self.u, &self.u_prev, self.visc, self.dt);

        std::mem::swap(&mut self.v_prev, &mut self.v);
        Self::diffuse(2, &mut self.v, &self.v_prev, self.visc, self.dt);

        Self::project(&mut self.u, &mut self.v, &mut self.u_prev, &mut self.v_prev);

        std::mem::swap(&mut self.u_prev, &mut self.u);
        std::mem::swap(&mut self.v_prev, &mut self.v);
        Self::advect(
            1,
            &mut self.u,
            &self.u_prev,
            &self.u_prev,
            &self.v_prev,
            self.dt,
        );
        Self::advect(
            2,
            &mut self.v,
            &self.v_prev,
            &self.u_prev,
            &self.v_prev,
            self.dt,
        );

        Self::project(&mut self.u, &mut self.v, &mut self.u_prev, &mut self.v_prev);

        // Update each dye channel: diffuse, advect, and fade.
        for (c, c_prev) in [
            (&mut self.r, &mut self.r_prev),
            (&mut self.g, &mut self.g_prev),
            (&mut self.b, &mut self.b_prev),
        ] {
            std::mem::swap(c_prev, c);
            Self::diffuse(0, c, c_prev, self.diff, self.dt);
            std::mem::swap(c_prev, c);
            Self::advect(0, c, c_prev, &self.u, &self.v, self.dt);

            // Fade dye gradually to prevent the screen from filling with color.
            for val in c.iter_mut() {
                *val *= 0.994;
            }
        }
    }
}

// Convert hue, saturation, and value to RGB.
fn hsv_to_rgb(h: f32, s: f32, v: f32) -> (f32, f32, f32) {
    let i = (h * 6.0).floor() as i32;
    let f = h * 6.0 - i as f32;
    let p = v * (1.0 - s);
    let q = v * (1.0 - f * s);
    let t = v * (1.0 - (1.0 - f) * s);
    match i % 6 {
        0 => (v, t, p),
        1 => (q, v, p),
        2 => (p, v, t),
        3 => (p, q, v),
        4 => (t, p, v),
        _ => (v, p, q),
    }
}

#[macroquad::main("Stable Fluids 2D")]
async fn main() {
    request_new_screen_size(720.0, 720.0);

    let mut fluid = FluidGrid::new(0.016, 0.00001, 0.00001);
    let mut image = Image::gen_image_color(N as u16, N as u16, BLACK);
    let texture = Texture2D::from_image(&image);
    texture.set_filter(FilterMode::Linear); // Smooth the grid when scaling it to the screen.

    let mut prev_mouse: Option<(f32, f32)> = None;
    let mut hue = 0.0;

    loop {
        // Inject velocity and dye while dragging the mouse.
        let (mx, my) = mouse_position();
        if is_mouse_button_down(MouseButton::Left) {
            let gx = ((mx / screen_width()) * N as f32) as i32;
            let gy = ((my / screen_height()) * N as f32) as i32;

            let (dx, dy) = if let Some((px, py)) = prev_mouse {
                (mx - px, my - py)
            } else {
                (0.0, 0.0)
            };

            hue = (hue + 0.006) % 1.0;
            let (cr, cg, cb) = hsv_to_rgb(hue, 0.85, 1.0);

            fluid.inject(gx, gy, dx, dy, cr, cg, cb);
            prev_mouse = Some((mx, my));
        } else {
            prev_mouse = None;
        }

        // Advance the fluid simulation.
        fluid.step();

        // Write the dye colors to the texture.
        for j in 1..=N {
            for i in 1..=N {
                let idx = ix(i, j);
                // Add the dark background and clamp to the displayable range.
                let r = (24.0 / 255.0 + fluid.r[idx]).min(1.0);
                let g = (24.0 / 255.0 + fluid.g[idx]).min(1.0);
                let b = (27.0 / 255.0 + fluid.b[idx]).min(1.0);
                image.set_pixel((i - 1) as u32, (j - 1) as u32, Color::new(r, g, b, 1.0));
            }
        }
        texture.update(&image);

        // Scale the texture to fill the window.
        draw_texture_ex(
            &texture,
            0.0,
            0.0,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(screen_width(), screen_height())),
                ..Default::default()
            },
        );

        next_frame().await
    }
}

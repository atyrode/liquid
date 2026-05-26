// World is a plane of width and height.
// Each frame, each particle compute its new velocity, then they all move simultaneously.
// new velocity get influenced by gravity and closeness to other Particles.

#[derive(Debug, Copy, Clone)]
pub struct Vec2f64 {
    pub x: f64,
    pub y: f64,
}

impl Vec2f64 {
    pub fn length(self) -> f64 {
        (self.x * self.x + self.y * self.y).sqrt()
    }
    pub fn dist_to(self, to: Vec2f64) -> f64 {
        (self - to).length()
    }
    pub fn normalized(self) -> Self {
        let length = self.length();
        if length == 0. {
            return Vec2f64 { x: 0.75, y: 0.75 };
        }
        self / length
    }
}

impl std::ops::Sub for Vec2f64 {
    type Output = Vec2f64;
    fn sub(self, rhs: Self) -> Self {
        Vec2f64 {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl std::ops::Add for Vec2f64 {
    type Output = Vec2f64;
    fn add(self, rhs: Self) -> Self {
        Vec2f64 {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl std::ops::AddAssign<Vec2f64> for Vec2f64 {
    fn add_assign(&mut self, rhs: Self) {
        self.x = self.x + rhs.x;
        self.y = self.y + rhs.y;
    }
}
impl std::ops::AddAssign<f64> for Vec2f64 {
    fn add_assign(&mut self, rhs: f64) {
        self.x = self.x + rhs;
        self.y = self.y + rhs;
    }
}

impl std::ops::Mul<f64> for Vec2f64 {
    type Output = Vec2f64;
    fn mul(self, rhs: f64) -> Self {
        Vec2f64 {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl std::ops::Div<f64> for Vec2f64 {
    type Output = Vec2f64;
    fn div(self, rhs: f64) -> Self {
        Vec2f64 {
            x: self.x / rhs,
            y: self.y / rhs,
        }
    }
}

#[derive(Debug)]
pub struct Particles {
    pub pos: Vec<Vec2f64>,
    pub velocity: Vec<Vec2f64>,
    pub board_width: f64,
    pub board_height: f64,
    pub time: f64,
}

use rand::RngExt;

const GRAVITY: f64 = -900.;

impl Particles {
    pub fn new(amount: usize, board_width: f64, board_height: f64) -> Particles {
        let mut pos = Vec::<Vec2f64>::with_capacity(amount);
        let mut velocity = Vec::<Vec2f64>::with_capacity(amount);
        let mut rng = rand::rng();
        for _ in 0..amount {
            pos.push(Vec2f64 {
                x: rng.random_range(0.0..board_width),
                y: rng.random_range(0.0..board_height),
            });
            velocity.push(Vec2f64 { x: 0., y: 0. });
        }
        Particles {
            pos,
            velocity,
            board_width,
            board_height,
            time: 0.,
        }
    }

    pub fn frame(&mut self, delta: f64) {
        self.time += delta;
        self.compute_new_velocity(delta);
        self.move_frame(delta);
    }

    fn compute_new_velocity(&mut self, delta: f64) {
        const REPULSION_DISTANCE: f64 = 30.0;
        for i in 0..self.velocity.len() {
            let rotated_gravity = Vec2f64 {
                x: (self.time * 0.1).sin(),
                y: (self.time * 0.1).cos(),
            } * GRAVITY;
            self.velocity[i] += rotated_gravity * delta;
            self.velocity[i] += noise2d_signed(self.pos[i].x, self.pos[i].y) * 10.;

            if self.pos[i].y < 20. {
                // TODO: RN Gravity is only countaeracted when going down, should depend on real gravity which can rotate !
                self.velocity[i].y += 35. + GRAVITY * delta;
            }
            if self.pos[i].y > self.board_height - 20. {
                self.velocity[i].y -= 35.;
            }
            if self.pos[i].x < 20. {
                self.velocity[i].x += 35.;
            }
            if self.pos[i].x > self.board_width - 20. {
                self.velocity[i].x -= 35.;
            }

            for j in (i + 1)..self.velocity.len() {
                if i == j {
                    continue;
                }
                let dist = self.pos[i].dist_to(self.pos[j]);
                if dist < REPULSION_DISTANCE {
                    let ratio = (REPULSION_DISTANCE - dist) / REPULSION_DISTANCE;
                    self.velocity[i] +=
                        (self.pos[i] - self.pos[j]).normalized() * delta * ratio * 150.;
                    self.velocity[j] +=
                        (self.pos[i] - self.pos[j]).normalized() * delta * -ratio * 150.;
                }
            }
        }
    }

    fn move_frame(&mut self, delta: f64) {
        for i in 0..self.velocity.len() {
            self.pos[i] += self.velocity[i] * delta;
            if self.pos[i].x < 0. {
                self.pos[i].x = 0.;
                self.velocity[i].x = 0.;
            }
            if self.pos[i].y < 0. {
                self.pos[i].y = 0.;
                self.velocity[i].y = 0.;
            }
            if self.pos[i].x > self.board_width {
                self.pos[i].x = self.board_width;
                self.velocity[i].x = 0.;
            }
            if self.pos[i].y > self.board_height {
                self.pos[i].y = self.board_height;
                self.velocity[i].y = 0.;
            }
        }
    }
}

fn noise2d(x: f64, y: f64) -> u32 {
    let xi = x.to_bits() as u32;
    let yi = y.to_bits() as u32;

    let mut n = xi.wrapping_mul(374761393);

    n = n.wrapping_add(yi.wrapping_mul(668265263));

    n = (n ^ (n >> 13)).wrapping_mul(1274126177);

    n ^ (n >> 16)
}
fn noise2d_f64(x: f64, y: f64) -> f64 {
    noise2d(x, y) as f64 / u32::MAX as f64
}
fn noise2d_signed(x: f64, y: f64) -> f64 {
    noise2d_f64(x, y) * 2.0 - 1.0
}

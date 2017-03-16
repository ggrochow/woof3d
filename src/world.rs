use vec2::Vec2;
use sdl2::pixels::Color;

static MOVE_SPEED_SCALING: f64 = 0.025;

#[derive(Debug)]
pub struct World {
    pub walls: Vec<Wall>,
    pub ground_color: Color,
    pub sky_color: Color,
    pub camera: Camera
}

impl World {
    pub fn move_forward(&mut self) {
        let move_vec = Vec2 {
            x: self.camera.theta.cos(),
            y: self.camera.theta.sin()
        }.multiply(MOVE_SPEED_SCALING);

        for wall in &self.walls {
            if let Some(_) = super::get_distance_to_line_line_intersection(&self.camera.p0, &move_vec, &wall.p0, &wall.p1) {
                return;
            }
        }

        self.camera.p0 = self.camera.p0.plus(&move_vec)
    }
}

impl Default for World {
    fn default() -> Self {
        World {
            walls: Vec::new(),
            ground_color: Color::RGB(60, 60, 60),
            sky_color: Color::RGB(135, 206, 235),
            camera: Camera::default(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Wall {
    pub p0: Vec2,
    pub p1: Vec2,
    pub color: Color,
}

impl Wall {
    pub fn new(x0: f64, y0: f64, x1: f64, y1: f64) -> Self {
        Wall {
            p0: Vec2 { x: x0, y: y0 },
            p1: Vec2 { x: x1, y: y1 },
            color: Color::RGB(100, 200, 150),
        }
    }

    pub fn new_usize(x0: usize, y0: usize, x1: usize, y1: usize) -> Self {
        Wall::new (x0 as f64, y0 as f64, x1 as f64, y1 as f64)
    }

}

#[derive(Debug)]
pub struct Camera {
    pub p0: Vec2,
    pub theta: f64,
    pub h_fov: f64,
    pub width: f64,
    pub height: f64,
    pub horizon: i32,
}

impl Default for Camera {
    fn default() -> Self {
        Camera {
            p0: Vec2{ x: 0.5, y: 0.5 },
            theta: 1.0,
            h_fov: 1.0,
            width: 800.0,
            height: 600.0,
            horizon: 300
        }
    }
}

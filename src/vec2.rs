#[derive(Debug, Clone)]
pub struct Vec2 {
    pub x: f64,
    pub y: f64,
}

impl Vec2 {
    pub fn plus(&self, other: &Vec2) -> Vec2 {
        Vec2 {
            x: self.x + other.x,
            y: self.y + other.y
        }
    }

    pub fn minus(&self, other: &Vec2) -> Vec2 {
        Vec2 {
            x: self.x - other.x,
            y: self.y - other.y
        }
    }

    pub fn multiply(&self, multiplier: f64) -> Vec2 {
        Vec2 {
            x: self.x * multiplier,
            y: self.y * multiplier
        }
    }

    pub fn cross(&self, other: &Vec2) -> f64 {
        self.x * other.y - self.y * other.x
    }

    pub fn distance(&self, other: &Vec2) -> f64 {
        ( (other.x - self.x).powi(2) + (other.y - self.y).powi(2) ).sqrt()
    }
}

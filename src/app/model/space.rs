use glam::{Vec3, Vec4};

pub struct Coord {
    pub coord: Vec3,
}

impl Coord {
    pub fn new(x: f32, y: f32, z: f32) -> Coord {
        Coord {
            coord: Vec3 { x, y, z },
        }
    }
}

pub struct Orientation {
    pub _direction: Vec4,
}

impl Orientation {
    pub fn null() -> Orientation {
        Orientation {
            _direction: Vec4::ZERO,
        }
    }
}

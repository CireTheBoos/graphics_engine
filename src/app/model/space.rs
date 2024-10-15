use glam::{Quat, Vec3};

pub struct Coord {
    pub coord: Vec3,
}

impl Coord {
    pub fn new(x: f32, y: f32, z: f32) -> Coord {
        Coord {
            coord: Vec3 { x, y, z },
        }
    }

    pub fn to_vec3(&self) -> Vec3 {
        self.coord
    }
}

pub struct Orientation {
    orientation: Quat,
}

impl Orientation {
    pub fn initial() -> Orientation {
        Orientation {
            orientation: Quat::IDENTITY,
        }
    }

    pub fn to_quat(&self) -> Quat {
        self.orientation
    }
}

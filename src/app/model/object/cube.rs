use super::super::space::{Coord, Orientation};

pub struct Cube {
    pub position: Coord,
    pub orientation: Orientation,
    pub size: f32,
}

impl Cube {
    pub fn new(position: Coord, orientation: Orientation, size: f32) -> Cube {
        Cube {
            position,
            orientation,
            size,
        }
    }

    pub fn new_unoriented(position: Coord, size: f32) -> Cube {
        Cube::new(position, Orientation::initial(), size)
    }
}

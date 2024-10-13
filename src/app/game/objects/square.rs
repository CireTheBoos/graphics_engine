use super::super::space::{Coord, Orientation};

pub struct Square {
    pub position: Coord,
    pub _orientation: Orientation,
    pub size: f32,
}

impl Square {
    pub fn new(position: Coord, size: f32) -> Square {
        Square {
            position,
            _orientation: Orientation::null(),
            size,
        }
    }
}

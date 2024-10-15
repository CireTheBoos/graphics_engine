use super::super::space::{Coord, Orientation};

pub struct Octahedron {
    pub position: Coord,
    pub orientation: Orientation,
    pub size: f32,
}

impl Octahedron {
    pub fn new(position: Coord, orientation: Orientation, size: f32) -> Octahedron {
        Octahedron {
            position,
            orientation,
            size,
        }
    }

    pub fn new_unoriented(position: Coord, size: f32) -> Octahedron {
        Octahedron::new(position, Orientation::initial(), size)
    }
}

use glam::Vec3;

use crate::app::game::objects::Square;

use super::{mesh::Mesh, vertex::Vertex};

pub fn from_square(square: &Square) -> Mesh {
    let top = square.position.coord + square.size * Vec3::X;
    let bottom = square.position.coord - square.size * Vec3::X;
    let right = square.position.coord + square.size * Vec3::Y;
    let left = square.position.coord - square.size * Vec3::Y;

    let top = Vertex::new(top, Vec3::X);
    let bottom = Vertex::new(bottom, Vec3::Y);
    let right = Vertex::new(right, Vec3::Z);
    let left = Vertex::new(left, Vec3::ONE);

    let vertices = vec![top, right, bottom, left];
    let indices = vec![0, 1, 2, 0, 2, 3];

    Mesh { vertices, indices }
}

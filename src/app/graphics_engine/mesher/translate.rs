use glam::Vec3;

use crate::app::model::object::Square;

use super::{
    mesh::{Mesh, Vertex},
    ToMesh,
};

impl ToMesh for Square {
    fn to_mesh(&self) -> Mesh {
        let top = self.position.coord + self.size * Vec3::X;
        let bottom = self.position.coord - self.size * Vec3::X;
        let right = self.position.coord + self.size * Vec3::Y;
        let left = self.position.coord - self.size * Vec3::Y;

        let top = Vertex::new(top, Vec3::X);
        let bottom = Vertex::new(bottom, Vec3::Y);
        let right = Vertex::new(right, Vec3::Z);
        let left = Vertex::new(left, Vec3::ONE);

        let vertices = vec![top, right, bottom, left];
        let indices = vec![0, 1, 2, 0, 2, 3];

        Mesh { vertices, indices }
    }
}

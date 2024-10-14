use glam::Vec3;

use crate::app::model::object::Square;

use super::{
    mesh::{Mesh, Vertex},
    ToMesh,
};

impl ToMesh for Square {
    fn to_mesh(&self) -> Mesh {
        let top = Vertex::new(self.position.coord + self.size * Vec3::X, Vec3::X);
        let bottom = Vertex::new(self.position.coord - self.size * Vec3::X, Vec3::X);
        let right = Vertex::new(self.position.coord + self.size * Vec3::Y, Vec3::Y);
        let left = Vertex::new(self.position.coord - self.size * Vec3::Y, Vec3::Y);
        let near = Vertex::new(self.position.coord + self.size * Vec3::Z, Vec3::Z);
        let far = Vertex::new(self.position.coord - self.size * Vec3::Z, Vec3::Z);

        let vertices = vec![top, bottom, right, left, near, far];
        let indices = vec![0,2,4,0,4,3,0,3,5,0,5,2,1,4,2,1,3,4,1,5,3,1,2,5];

        Mesh { vertices, indices }
    }
}

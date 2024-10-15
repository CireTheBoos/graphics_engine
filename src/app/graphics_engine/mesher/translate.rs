use glam::{Mat4, Vec3};

use crate::app::model::object::{Cube, Octahedron};

use super::{Mesh, ToMesh, Vertex};

impl ToMesh for Octahedron {
    fn transform(&self) -> Mat4 {
        glam::Mat4::from_scale_rotation_translation(
            Vec3::ONE * self.size,
            self.orientation.to_quat(),
            self.position.to_vec3(),
        )
    }
    fn mesh(&self) -> Mesh {
        let top = Vertex::new(Vec3::X, Vec3::X);
        let bottom = Vertex::new(-Vec3::X, Vec3::X);
        let right = Vertex::new(Vec3::Y, Vec3::Y);
        let left = Vertex::new(-Vec3::Y, Vec3::Y);
        let near = Vertex::new(Vec3::Z, Vec3::Z);
        let far = Vertex::new(-Vec3::Z, Vec3::Z);

        let vertices = vec![top, bottom, right, left, near, far];
        let indices = vec![
            0, 2, 4, 0, 4, 3, 0, 3, 5, 0, 5, 2, 1, 4, 2, 1, 3, 4, 1, 5, 3, 1, 2, 5,
        ];

        Mesh { vertices, indices }
    }
}

impl ToMesh for Cube {
    fn transform(&self) -> Mat4 {
        glam::Mat4::from_scale_rotation_translation(
            Vec3::ONE * self.size,
            self.orientation.to_quat(),
            self.position.to_vec3(),
        )
    }
    fn mesh(&self) -> Mesh {
        let top = Vertex::new(Vec3::X, Vec3::X);
        let bottom = Vertex::new(-Vec3::X, Vec3::X);
        let right = Vertex::new(Vec3::Y, Vec3::Y);
        let left = Vertex::new(-Vec3::Y, Vec3::Y);
        let near = Vertex::new(Vec3::Z, Vec3::Z);
        let far = Vertex::new(-Vec3::Z, Vec3::Z);

        let vertices = vec![top, bottom, right, left, near, far];
        let indices = vec![
            0, 2, 4, 0, 4, 3, 0, 3, 5, 0, 5, 2, 1, 4, 2, 1, 3, 4, 1, 5, 3, 1, 2, 5,
        ];

        Mesh { vertices, indices }
    }
}

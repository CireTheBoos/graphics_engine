use glam::Mat4;

use super::Vertex;

pub trait ToMesh {
    fn transform(&self) -> Mat4;
    fn mesh(&self) -> Mesh;
}

pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

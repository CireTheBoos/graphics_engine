mod mesh;
mod translate;

pub const MAX_VERTICES: u64 = 12;
pub const MAX_INDICES: u64 = 32;

pub use mesh::{Mesh, Vertex};

pub trait ToMesh {
    fn to_mesh(&self) -> Mesh;
}

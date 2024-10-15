mod mesh;
mod translate;
mod vertex;

pub const MAX_VERTICES: u64 = 12;
pub const MAX_INDICES: u64 = 32;

pub use mesh::{Mesh, ToMesh};
pub use vertex::Vertex;

use shaderc::CompilationArtifact;

use super::shaders;

pub struct RendererPipeline {
    vertex: CompilationArtifact,
    fragment: CompilationArtifact,
}

impl RendererPipeline {
    pub fn new() -> RendererPipeline {
        // compiling shaders
        let vertex = shaders::vertex();
        let fragment = shaders::fragment();
        RendererPipeline {
            vertex,
            fragment,
        }
    }

    pub fn vertex_binary(&self) -> &[u32] {
        self.vertex.as_binary()
    }

    pub fn fragment_binary(&self) -> &[u32] {
        self.fragment.as_binary()
    }
}

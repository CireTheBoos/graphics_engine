use shaderc::{self, CompilationArtifact};

const VERTEX_SHADER_SRC: &str = "
#version 450

vec2 positions[3] = vec2[](
    vec2(0.0, -0.5),
    vec2(0.5, 0.5),
    vec2(-0.5, 0.5)
);

void main() {
    gl_Position = vec4(positions[gl_VertexIndex], 0.0, 1.0);
}
";

pub fn vertex() -> CompilationArtifact {
    let compiler = shaderc::Compiler::new().unwrap();
    // options seems interesting
    // let mut options = shaderc::CompileOptions::new().unwrap();
    // options.add_macro_definition("EP", Some("main"));
    let binary_result = compiler
        .compile_into_spirv(
            VERTEX_SHADER_SRC,
            shaderc::ShaderKind::Vertex,
            "vertex.glsl",
            "main",
            None,
        )
        .unwrap();

    binary_result
}

const FRAGMENT_SHADER_SRC: &str = " 
#version 450

layout(location = 0) out vec4 outColor;

void main() {
    outColor = vec4(1.0, 0.0, 0.0, 1.0);
}
";

pub fn fragment() -> CompilationArtifact {
    let compiler = shaderc::Compiler::new().unwrap();
    let binary_result = compiler
        .compile_into_spirv(
            FRAGMENT_SHADER_SRC,
            shaderc::ShaderKind::Fragment,
            "fragment.glsl",
            "main",
            None,
        )
        .unwrap();

    binary_result
}

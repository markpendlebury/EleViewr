// Vertex shader
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
};

// Uniform buffer for aspect ratio preservation
struct Uniforms {
    screen_aspect: f32,
    image_aspect: f32,
    scale_factor: f32,
};

@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;
@group(0) @binding(2)
var<uniform> uniforms: Uniforms;

@vertex
fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;

    // Create a fullscreen quad using 6 vertices
    var positions = array<vec2<f32>, 6>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(1.0, -1.0),
        vec2<f32>(-1.0, 1.0),
        vec2<f32>(-1.0, 1.0),
        vec2<f32>(1.0, -1.0),
        vec2<f32>(1.0, 1.0),
    );
    
    var texcoords = array<vec2<f32>, 6>(
        vec2<f32>(0.0, 1.0),
        vec2<f32>(1.0, 1.0),
        vec2<f32>(0.0, 0.0),
        vec2<f32>(0.0, 0.0),
        vec2<f32>(1.0, 1.0),
        vec2<f32>(1.0, 0.0),
    );
    
    // Apply aspect ratio correction to vertex positions
    var pos = positions[in_vertex_index];
    
    // Adjust position based on aspect ratio
    if (uniforms.screen_aspect > uniforms.image_aspect) {
        // Screen is wider than the image, adjust x-coordinate
        pos.x = pos.x * (uniforms.image_aspect / uniforms.screen_aspect) * uniforms.scale_factor;
    } else {
        // Screen is taller than the image, adjust y-coordinate
        pos.y = pos.y * (uniforms.screen_aspect / uniforms.image_aspect) * uniforms.scale_factor;
    }
    
    out.clip_position = vec4<f32>(pos, 0.0, 1.0);
    out.tex_coords = texcoords[in_vertex_index];
    
    return out;
}

// Fragment shader
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(t_diffuse, s_diffuse, in.tex_coords);
}
#import bevy_pbr::{
    pbr_fragment::pbr_input_from_standard_material,
    pbr_functions::alpha_discard,
}

#ifdef PREPASS_PIPELINE
#import bevy_pbr::{
    prepass_io::{VertexOutput, FragmentOutput},
    pbr_deferred_functions::deferred_output,
}
#else
#import bevy_pbr::{
    forward_io::{VertexOutput, FragmentOutput},
    pbr_functions::{apply_pbr_lighting, main_pass_post_lighting_processing},
}
#endif

struct MaterialSettings {
    flip: u32,
    angle: f32,
}

@group(2) @binding(100)
var<uniform> material: MaterialSettings ;

const TAU:f32 =  6.28318530718;

/// Clockwise by `theta`
fn rotate2D(theta: f32) -> mat2x2<f32> {
    let c = cos(theta);
    let s = sin(theta);
    return mat2x2<f32>(c, s, -s, c);
}

@fragment
fn fragment(
    in: VertexOutput,
    @builtin(front_facing) is_front: bool,
) -> FragmentOutput {
    var uvs = in.uv;

    if material.flip == 1 {
        uvs.x = 1.0 - uvs.x;
    }
    
    uvs -= 0.5;
    uvs *= rotate2D(material.angle - TAU);
    uvs += 0.5;

    var modified_input = in;
    modified_input.uv = uvs;

    var pbr_input = pbr_input_from_standard_material(modified_input, is_front);

#ifdef PREPASS_PIPELINE
    // in deferred mode we can't modify anything after that, as lighting is run in a separate fullscreen shader.
    let out = deferred_output(in, pbr_input);
#else
    var out: FragmentOutput;
    // apply lighting
    out.color = apply_pbr_lighting(pbr_input);
#endif

    return out;
}

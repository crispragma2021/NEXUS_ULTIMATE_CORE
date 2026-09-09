// SDF Shader mínimo para demo_userland_sdf (restaurado — el original se perdió en la migración)
struct Uniforms {
    time: f32,
    resolution: vec2<f32>,
    _pad: vec2<f32>,
}

@group(0) @binding(0) var<uniform> uniforms: Uniforms;

struct VsOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vi: u32) -> VsOut {
    // Fullscreen triangle
    var pos = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(3.0, -1.0),
        vec2<f32>(-1.0, 3.0),
    );
    var out: VsOut;
    out.pos = vec4<f32>(pos[vi], 0.0, 1.0);
    out.uv = pos[vi] * 0.5 + 0.5;
    return out;
}

// SDF del círculo
fn sd_circle(p: vec2<f32>, r: f32) -> f32 {
    return length(p) - r;
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let uv = in.uv * 2.0 - 1.0;
    let aspect = uniforms.resolution.x / max(uniforms.resolution.y, 1.0);
    let p = vec2<f32>(uv.x * aspect, uv.y);

    // Círculo pulsante con SDF
    let r = 0.35 + 0.1 * sin(uniforms.time * 2.0);
    let d = sd_circle(p, r);
    let color = vec3<f32>(0.9, 0.3, 0.2) * (1.0 - smoothstep(0.0, 0.02, d));

    // Fondo oscuro
    let bg = vec3<f32>(0.05, 0.05, 0.1);
    return vec4<f32>(mix(bg, color, step(d, 0.0)) + vec3<f32>(0.0), 1.0);
}

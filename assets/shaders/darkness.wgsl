struct Lamp {
    a: vec2<f32>,
    b: vec2<f32>,
    color: vec4<f32>,
    radius: f32,
};

struct Lighting {
    view_min: vec2<f32>,
    view_size: vec2<f32>,
    viewport: vec2<f32>,
    count: u32,
    ambient: vec4<f32>,
    lamps: array<Lamp, 32>,
};

@group(2) @binding(0) var<uniform> lighting: Lighting;

const PIXEL: f32 = 2.0;
const HOT: f32 = 60.0;
const GLOW: f32 = 0.06;

fn segment_distance(p: vec2<f32>, a: vec2<f32>, b: vec2<f32>) -> f32 {
    let ab = b - a;
    let t = clamp(dot(p - a, ab) / max(dot(ab, ab), 0.0001), 0.0, 1.0);
    return distance(p, a + ab * t);
}

fn noise(p: vec2<f32>) -> f32 {
    return fract(sin(dot(p, vec2<f32>(12.9898, 78.233))) * 43758.5453) - 0.5;
}

@fragment
fn fragment(@builtin(position) frag: vec4<f32>) -> @location(0) vec4<f32> {
    let screen_uv = frag.xy / lighting.viewport;
    let world_exact = lighting.view_min + vec2<f32>(screen_uv.x, 1.0 - screen_uv.y) * lighting.view_size;
    let world = (floor(world_exact / PIXEL) + 0.5) * PIXEL;

    var dark_left = 1.0;
    var glow = vec3<f32>(0.0);
    for (var i = 0u; i < lighting.count; i = i + 1u) {
        let lamp = lighting.lamps[i];
        let t = segment_distance(world, lamp.a, lamp.b) / max(lamp.radius, 0.0001);
        if (t >= 1.0) {
            continue;
        }
        let reach = 1.0 - t * t;
        dark_left = dark_left * (1.0 - clamp(reach * lamp.color.a, 0.0, 1.0));
        let hot = reach * reach / (1.0 + HOT * t * t);
        glow = glow + lamp.color.rgb * hot * lamp.color.a;
    }

    let dark = lighting.ambient.a * dark_left;
    let strength = max(glow.r, max(glow.g, glow.b));
    let tint = min(strength, 1.0) * GLOW;
    let hue = glow / max(strength, 0.0001);
    let alpha = clamp(dark + tint + noise(frag.xy) / 255.0, 0.0, 1.0);
    let rgb = (lighting.ambient.rgb * dark + hue * tint) / max(dark + tint, 0.0001);
    return vec4<f32>(rgb, alpha);
}

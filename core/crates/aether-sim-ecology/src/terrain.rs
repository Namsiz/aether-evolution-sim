//! terrain.rs — bit-exact port of `living/src/world.js` terrain functions.
//!
//! Only the subset required by the ecology sim is ported: noise2/fbm2/ridgedFbm/
//! terrainHeight/slopeAt/plantSuitability, plus smoothstep/clamp/lerp helpers.
//! The Perlin permutation table is hardcoded to the exact 512-byte sequence the
//! JS oracle derives from `mulberry32(1337)` at module-load time (world.js L33-47);
//! deriving it at runtime would burn 256 RNG draws and force a reordered PRNG contract.

pub const WORLD_SIZE: f64 = 1600.0;
pub const WORLD_HALF: f64 = 800.0;
pub const WATER_LEVEL: f64 = 0.0;
pub const SIM_RADIUS: f64 = 560.0;

pub fn smoothstep(a: f64, b: f64, x: f64) -> f64 {
    let t = ((x - a) / (b - a)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

pub fn clamp(v: f64, a: f64, b: f64) -> f64 {
    if v < a { a } else if v > b { b } else { v }
}

pub fn lerp(a: f64, b: f64, t: f64) -> f64 {
    a + (b - a) * t
}

/// Permutation table, seeded identically to `world.js` `buildPerm()` IIFE (mulberry32(1337)).
const PERM: [u8; 512] = [
    238, 184, 179, 185, 206, 6, 115, 50, 64, 232, 152, 76, 233, 174, 28, 199,
    106, 0, 156, 235, 122, 123, 159, 138, 23, 38, 100, 66, 18, 35, 88, 145,
    218, 249, 77, 119, 146, 97, 170, 203, 31, 173, 42, 139, 210, 16, 71, 81,
    140, 128, 36, 91, 90, 130, 168, 244, 19, 89, 207, 57, 40, 236, 157, 253,
    3, 212, 118, 102, 133, 125, 92, 39, 200, 220, 107, 254, 177, 37, 1, 24,
    13, 75, 72, 5, 126, 117, 182, 61, 70, 163, 11, 161, 209, 112, 127, 79,
    34, 246, 248, 21, 116, 86, 195, 137, 142, 85, 171, 113, 63, 186, 214, 73,
    158, 132, 12, 217, 189, 228, 219, 190, 197, 44, 194, 187, 243, 87, 56, 227,
    14, 176, 149, 53, 188, 237, 165, 94, 22, 80, 151, 183, 224, 17, 204, 148,
    15, 208, 60, 68, 164, 245, 103, 83, 240, 74, 4, 9, 172, 129, 78, 160,
    29, 230, 175, 147, 193, 211, 216, 58, 82, 99, 181, 166, 144, 180, 213, 55,
    191, 154, 201, 226, 239, 109, 110, 242, 202, 225, 155, 234, 33, 10, 251, 114,
    101, 104, 143, 196, 215, 223, 65, 51, 26, 25, 52, 32, 8, 141, 198, 222,
    98, 93, 120, 62, 121, 69, 54, 30, 2, 59, 84, 231, 124, 7, 178, 45,
    229, 41, 153, 27, 43, 167, 169, 250, 67, 105, 241, 255, 20, 150, 221, 135,
    192, 49, 96, 111, 46, 252, 247, 134, 136, 131, 95, 108, 162, 205, 48, 47,
    238, 184, 179, 185, 206, 6, 115, 50, 64, 232, 152, 76, 233, 174, 28, 199,
    106, 0, 156, 235, 122, 123, 159, 138, 23, 38, 100, 66, 18, 35, 88, 145,
    218, 249, 77, 119, 146, 97, 170, 203, 31, 173, 42, 139, 210, 16, 71, 81,
    140, 128, 36, 91, 90, 130, 168, 244, 19, 89, 207, 57, 40, 236, 157, 253,
    3, 212, 118, 102, 133, 125, 92, 39, 200, 220, 107, 254, 177, 37, 1, 24,
    13, 75, 72, 5, 126, 117, 182, 61, 70, 163, 11, 161, 209, 112, 127, 79,
    34, 246, 248, 21, 116, 86, 195, 137, 142, 85, 171, 113, 63, 186, 214, 73,
    158, 132, 12, 217, 189, 228, 219, 190, 197, 44, 194, 187, 243, 87, 56, 227,
    14, 176, 149, 53, 188, 237, 165, 94, 22, 80, 151, 183, 224, 17, 204, 148,
    15, 208, 60, 68, 164, 245, 103, 83, 240, 74, 4, 9, 172, 129, 78, 160,
    29, 230, 175, 147, 193, 211, 216, 58, 82, 99, 181, 166, 144, 180, 213, 55,
    191, 154, 201, 226, 239, 109, 110, 242, 202, 225, 155, 234, 33, 10, 251, 114,
    101, 104, 143, 196, 215, 223, 65, 51, 26, 25, 52, 32, 8, 141, 198, 222,
    98, 93, 120, 62, 121, 69, 54, 30, 2, 59, 84, 231, 124, 7, 178, 45,
    229, 41, 153, 27, 43, 167, 169, 250, 67, 105, 241, 255, 20, 150, 221, 135,
    192, 49, 96, 111, 46, 252, 247, 134, 136, 131, 95, 108, 162, 205, 48, 47,
];

#[inline]
fn fade(t: f64) -> f64 {
    t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
}

#[inline]
fn grad(hash: u8, x: f64, y: f64) -> f64 {
    match hash & 7 {
        0 => x + y,
        1 => -x + y,
        2 => x - y,
        3 => -x - y,
        4 => x,
        5 => -x,
        6 => y,
        _ => -y,
    }
}

pub fn noise2(x: f64, y: f64) -> f64 {
    // JS: xi = Math.floor(x) & 255 — the & 255 wraps negative floats' truncation toward zero
    // through bitwise AND on int32, so we mirror via ((floor as i64) & 255).
    let fx = x.floor();
    let fy = y.floor();
    let xi = ((fx as i64) & 255) as usize;
    let yi = ((fy as i64) & 255) as usize;
    let xf = x - fx;
    let yf = y - fy;
    let u = fade(xf);
    let v = fade(yf);
    let aa = PERM[PERM[xi] as usize + yi] as i32;
    let ab = PERM[PERM[xi] as usize + yi + 1] as i32;
    let ba = PERM[PERM[xi + 1] as usize + yi] as i32;
    let bb = PERM[PERM[xi + 1] as usize + yi + 1] as i32;
    let x1 = lerp(grad(aa as u8, xf, yf), grad(ba as u8, xf - 1.0, yf), u);
    let x2 = lerp(grad(ab as u8, xf, yf - 1.0), grad(bb as u8, xf - 1.0, yf - 1.0), u);
    lerp(x1, x2, v) * 0.70710678
}

pub fn fbm2(x: f64, y: f64, octaves: usize) -> f64 {
    fbm2_full(x, y, octaves, 2.0, 0.5)
}

pub fn fbm2_full(x: f64, y: f64, octaves: usize, lacunarity: f64, gain: f64) -> f64 {
    let mut amp = 0.5;
    let mut freq = 1.0;
    let mut sum = 0.0;
    for _ in 0..octaves {
        sum += noise2(x * freq, y * freq) * amp;
        freq *= lacunarity;
        amp *= gain;
    }
    sum
}

pub fn ridged_fbm(x: f64, y: f64, octaves: usize) -> f64 {
    let mut amp = 0.55;
    let mut freq = 1.0;
    let mut sum = 0.0;
    let mut norm = 0.0;
    for _ in 0..octaves {
        let n = 1.0 - (noise2(x * freq, y * freq) * 2.0).abs();
        sum += n * n * amp;
        norm += amp;
        freq *= 2.03;
        amp *= 0.5;
    }
    sum / norm
}


/// Bit-exact reimplementation of V8's `Math.hypot(x, y)` for two arguments.
/// V8 (Node.js) uses `max * sqrt(1 + (min/max)^2)` — NOT glibc's `hypot()`, which
/// differs in the last bit for many inputs. See node v22.22.1 x86_64-linux.
pub fn js_hypot(x: f64, y: f64) -> f64 {
    let ax = x.abs();
    let ay = y.abs();
    let (a, b) = if ax >= ay { (ax, ay) } else { (ay, ax) };
    if a == 0.0 { return 0.0; }
    let r = b / a;
    a * (1.0 + r * r).sqrt()
}

pub fn terrain_height(x: f64, z: f64) -> f64 {
    let d = js_hypot(x, z);
    let ang = aether_libm::atan2(z, x);
    let shore = smoothstep(105.0, 240.0, d);

    let mut h = -9.5 + 12.5 * shore;

    let roll = fbm2(x * 0.0055 + 13.1, z * 0.0055 - 4.7, 4) * 2.0;
    h += roll * 5.5 * smoothstep(120.0, 255.0, d);

    let m = smoothstep(330.0, 730.0, d);
    let ridge = ridged_fbm(x * 0.0028 + 2.2, z * 0.0028 - 1.7, 5);
    let dir = 0.48 + 0.52 * (0.5 + 0.5 * aether_libm::cos(ang - 2.15));
    h += aether_libm::pow(m, 1.35) * (26.0 + 165.0 * ridge) * dir;

    let crag = smoothstep(55.0, 135.0, h) * (fbm2(x * 0.021, z * 0.021, 3) * 2.0) * 13.0;
    h += crag;

    h += smoothstep(650.0, 900.0, d) * 24.0;
    h
}

pub fn slope_at(x: f64, z: f64) -> f64 {
    let e = 3.0;
    let dx = terrain_height(x + e, z) - terrain_height(x - e, z);
    let dz = terrain_height(x, z + e) - terrain_height(x, z - e);
    js_hypot(dx, dz) / (2.0 * e)
}

pub fn plant_suitability(x: f64, z: f64) -> f64 {
    let h = terrain_height(x, z);
    if h < WATER_LEVEL + 1.2 || h > 78.0 {
        return 0.0;
    }
    let s = slope_at(x, z);
    let wet = smoothstep(230.0, 120.0, js_hypot(x, z)) * 0.35 + 0.65;
    clamp(
        smoothstep(WATER_LEVEL + 1.2, WATER_LEVEL + 7.0, h)
            * (1.0 - smoothstep(0.42, 0.85, s))
            * (1.0 - smoothstep(58.0, 88.0, h))
            * wet,
        0.0,
        1.0,
    )
}

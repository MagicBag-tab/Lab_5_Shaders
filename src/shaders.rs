use crate::triangle::Vec3;
type Color = [f64; 3];
pub(crate) fn mix(a: Color, b: Color, t: f64) -> Color {
    let t = t.clamp(0.0, 1.0);
    std::array::from_fn(|i| a[i] * (1.0 - t) + b[i] * t)
}
pub(crate) fn smooth(a: f64, b: f64, x: f64) -> f64 {
    let t = ((x - a) / (b - a)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}
fn hash(x: i64, y: i64, z: i64) -> f64 {
    let mut h = (x as u64).wrapping_mul(374761393)
        ^ (y as u64).wrapping_mul(668265263)
        ^ (z as u64).wrapping_mul(2147483647);
    h = (h ^ (h >> 13)).wrapping_mul(1274126177);
    ((h ^ (h >> 16)) & 0xffffff) as f64 / 16777215.0
}
pub(crate) fn noise(p: Vec3) -> f64 {
    let base = [p.x.floor() as i64, p.y.floor() as i64, p.z.floor() as i64];
    let f =
        [p.x - p.x.floor(), p.y - p.y.floor(), p.z - p.z.floor()].map(|t| t * t * (3.0 - 2.0 * t));
    let mut value = 0.0;
    for x in 0..2 {
        for y in 0..2 {
            for z in 0..2 {
                let w = |i: usize, n| if n == 1 { f[i] } else { 1.0 - f[i] };
                value += hash(base[0] + x, base[1] + y, base[2] + z) * w(0, x) * w(1, y) * w(2, z);
            }
        }
    }
    value
}
const CREAM: Color = [1.0, 0.94, 0.79];
const GOLD: Color = [1.0, 0.72, 0.30];
const MINT: Color = [0.49, 0.94, 0.82];
const LAVENDER: Color = [0.66, 0.48, 0.86];
const PINK: Color = [0.98, 0.56, 0.77];

fn craters(p: Vec3) -> (f64, f64) {
    let p = p.normalize();
    let mut basin: f64 = 0.0;
    let mut rim: f64 = 0.0;
    for (x, y, z, r) in [
        (-0.45, 0.35, 0.82, 0.18),
        (0.3, -0.4, 0.86, 0.14),
        (0.5, 0.5, 0.7, 0.1),
        (-0.2, -0.65, 0.73, 0.12),
        (0.8, -0.15, 0.58, 0.16),
        (-0.8, -0.25, 0.54, 0.08),
        (0.3, 0.6, -0.74, 0.2),
        (-0.5, -0.4, -0.77, 0.15),
    ] {
        let d = p.sub(Vec3::new(x, y, z).normalize()).length() / r;
        basin = basin.max(1.0 - smooth(0.65, 0.95, d));
        rim = rim.max(1.0 - smooth(0.06, 0.18, (d - 1.0).abs()));
    }
    (basin, rim)
}

pub fn shade(design: usize, part: u8, local: Vec3, normal: Vec3, position: Vec3) -> Option<u32> {
    let p = local;
    let mut n = normal.normalize();
    let view = Vec3::new(0.0, 0.0, 5.0).sub(position).normalize();
    if n.dot(view) < 0.0 {
        n = n.scale(-1.0);
    }
    let light = Vec3::new(-0.5, 0.7, 1.0).normalize();
    let emissive = design == 0;
    let base = if part == 1 || part == 3 {
        let r = (p.x * p.x + p.z * p.z).sqrt();
        let stripes = (r * 170.0).sin() * 0.5 + 0.5;
        if part == 3 && stripes < 0.12 {
            return None;
        }
        mix(mix(GOLD, MINT, stripes), CREAM, 0.25)
    } else if part == 2 {
        let q = p.sub(Vec3::new(1.25, 0.74, 0.14)).scale(1.0 / 0.21);
        let craters = noise(q.scale(9.0));
        let c = mix(
            [0.42, 0.37, 0.59],
            [0.89, 0.84, 0.96],
            smooth(0.3, 0.7, craters),
        );
        let (basin, rim) = self::craters(q);
        mix(mix(c, [0.32, 0.28, 0.43], basin * 0.7), CREAM, rim * 0.45)
    } else {
        match design {
            0 => {
                let cells = noise(p.scale(12.0));
                let regions = noise(p.scale(3.5));
                let filaments = 1.0 - smooth(0.015, 0.055, (cells - 0.5).abs());
                let c = mix([1.0, 0.67, 0.48], [0.96, 0.29, 0.37], regions);
                let c = mix(c, GOLD, smooth(0.52, 0.76, cells) * 0.8);
                mix(c, CREAM, filaments * 0.7)
            }
            1 => {
                let minerals = noise(p.scale(4.5));
                let crater = noise(p.scale(22.0));
                let veins = 1.0 - smooth(0.012, 0.05, (noise(p.scale(8.0)) - 0.5).abs());
                let c = mix(LAVENDER, [0.31, 0.17, 0.49], smooth(0.35, 0.7, minerals));
                let c = mix(c, [0.29, 0.24, 0.40], smooth(0.64, 0.76, crater) * 0.65);
                let c = mix(c, MINT, veins * 0.7);
                let (basin, rim) = craters(p);
                let c = mix(mix(c, [0.31, 0.23, 0.43], basin * 0.65), CREAM, rim * 0.3);
                mix(c, GOLD, smooth(0.79, 0.89, noise(p.scale(70.0))))
            }
            _ => {
                let warp = noise(p.scale(5.0));
                let latitude = p.y * 19.0 + (warp - 0.5) * 4.0;
                let bands = latitude.sin() * 0.5 + 0.5;
                let c = mix(LAVENDER, PINK, bands);
                let c = mix(
                    c,
                    [0.41, 0.66, 0.9],
                    smooth(0.65, 0.96, (latitude * 0.57).sin()),
                );
                let c = mix(c, CREAM, smooth(0.72, 0.95, bands) * 0.8);
                let storm = ((p.x - 0.35) / 0.25).powi(2) + ((p.y + 0.22) / 0.12).powi(2);
                let c = mix(
                    c,
                    [0.95, 0.65, 0.46],
                    (1.0 - smooth(0.6, 1.2, storm)) * smooth(0.0, 0.5, p.z),
                );
                mix(c, CREAM, smooth(0.65, 0.84, noise(p.scale(17.0))) * 0.4)
            }
        }
    };
    let diffuse = if emissive {
        1.0
    } else {
        0.52 + 0.48 * n.dot(light).max(0.0)
    };
    let half = light.add(view).normalize();
    let spec = n.dot(half).max(0.0).powi(90);
    let fresnel = (1.0 - n.dot(view).max(0.0)).powi(3);
    let c = base.map(|v| v * diffuse);
    let c = mix(
        c,
        mix(MINT, PINK, 0.5 + 0.5 * (p.y * 4.0).sin()),
        fresnel * 0.6,
    );
    let c = mix(c, [1.0, 1.0, 1.0], (spec * 0.92 + fresnel * 0.22).min(1.0));
    let c = c.map(|v| (v.clamp(0.0, 1.0) * 255.0).round() as u32);
    Some(c[0] << 16 | c[1] << 8 | c[2])
}

pub fn interpolate(values: [Vec3; 3], w: [f64; 3]) -> Vec3 {
    values[0]
        .scale(w[0])
        .add(values[1].scale(w[1]))
        .add(values[2].scale(w[2]))
}

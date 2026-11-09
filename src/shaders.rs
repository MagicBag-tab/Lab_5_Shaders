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
    let emissive = design == 0 || part == 7;
    let base = if part == 4 {
        // Iris uses its own circular mesh; positions are scene-local XY.
        let radius = (p.x * p.x + p.y * p.y).sqrt() / 0.435;
        let angle = p.y.atan2(p.x);
        let filaments =
            (angle * 72.0 + radius * 18.0 + (radius * 28.0).sin() * 1.8).sin() * 0.5 + 0.5;
        let c = mix(MINT, [0.49, 0.25, 0.75], filaments);
        let limbus = smooth(0.83, 1.0, radius);
        let c = mix(c, [0.26, 0.18, 0.45], limbus * 0.8);
        let gold = smooth(0.985, 1.0, (angle * 25.0 + radius * 7.0).sin()) * 0.55;
        let c = mix(c, GOLD, gold);
        mix([0.055, 0.025, 0.12], c, smooth(0.23, 0.28, radius))
    } else if part == 5 {
        let flow = (p.x * 3.0 + p.y * 2.0).sin() * 0.5 + 0.5;
        mix(mix(GOLD, PINK, flow * 0.7), CREAM, 0.22)
    } else if matches!(part, 6 | 8 | 9) {
        let c = mix(GOLD, PINK, 0.5 + 0.5 * (p.y * 12.0 + p.x * 9.0).sin());
        mix(c, CREAM, smooth(0.3, 0.85, noise(p.scale(10.0))) * 0.55)
    } else if part == 7 {
        let waves = (p.y * 22.0 + noise(p.scale(13.0)) * 4.0).sin() * 0.5 + 0.5;
        let c = mix([0.93, 0.57, 0.8], CREAM, waves);
        mix(c, GOLD, smooth(0.83, 1.0, waves) * 0.5)
    } else if part == 1 || part == 3 {
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
                let c = mix(c, MINT, veins * 0.22);
                let (basin, rim) = craters(p);
                let c = mix(mix(c, [0.31, 0.23, 0.43], basin * 0.65), CREAM, rim * 0.3);
                mix(c, GOLD, smooth(0.79, 0.89, noise(p.scale(70.0))) * 0.3)
            }
            2 => {
                let warp = crate::techniques::fbm(p.scale(4.0));
                let storm_distance = ((p.x - 0.35) / 0.28).powi(2) + ((p.y + 0.22) / 0.14).powi(2);
                let eddy = (1.0 - smooth(0.0, 3.0, storm_distance))
                    * ((p.y + 0.22).atan2(p.x - 0.35) * 3.0 + storm_distance * 4.0).sin();
                let latitude = p.y * 22.0 + (warp - 0.5) * 7.0 + eddy * 1.7;
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
                mix(
                    c,
                    CREAM,
                    smooth(0.52, 0.74, crate::techniques::fbm(p.scale(13.0))) * 0.55,
                )
            }
            3 => {
                let regions = noise(p.scale(2.5));
                let c = mix(LAVENDER, PINK, regions);
                let vein = 1.0 - smooth(0.009, 0.028, (noise(p.scale(6.0)) - 0.5).abs());
                let c = mix(c, GOLD, vein * 0.75);
                let blush = 0.5 + 0.5 * (p.y * 6.0 + p.x * 2.0).sin();
                mix(c, CREAM, blush * 0.12)
            }
            4 => {
                // Continuous coordinate field wraps the curved volume, including its back.
                let wave =
                    p.y * 17.0 + (p.x * 5.0 + p.z * 3.0).sin() * 2.3 + noise(p.scale(3.0)) * 2.0;
                let band = wave.sin();
                let c = mix(LAVENDER, PINK, 0.5 + 0.5 * (wave * 0.47).sin());
                let paths = 1.0 - smooth(0.24, 0.4, band.abs());
                let borders = (1.0 - smooth(0.025, 0.095, (band.abs() - 0.4).abs())) * 0.7;
                let c = mix(c, MINT, paths * 0.85);
                let c = mix(c, CREAM, borders);
                let gilding = 1.0 - smooth(0.01, 0.035, (band - 0.72).abs());
                mix(c, GOLD, gilding * 0.55)
            }
            5 => {
                let r = (p.x * p.x + p.y * p.y).sqrt();
                let angle = p.y.atan2(p.x);
                // Integer angular frequency keeps the atan2 seam continuous.
                let swirl = angle * 5.0 - r * 13.0 + p.z * 4.0 + noise(p.scale(3.0)) * 1.5;
                let wave = swirl.sin();
                let c = mix(LAVENDER, PINK, wave * 0.5 + 0.5);
                let cream = smooth(0.55, 0.85, wave);
                let c = mix(c, CREAM, cream * 0.75);
                let gold = 1.0 - smooth(0.015, 0.065, (wave + 0.4).abs());
                let c = mix(c, GOLD, gold * 0.8);
                mix(c, MINT, smooth(0.7, 0.9, noise(p.scale(8.0))) * 0.25)
            }
            _ => LAVENDER,
        }
    };
    let rocky = design == 1;
    let gaseous = design == 2 && part == 0;
    let diffuse = if emissive {
        1.0
    } else if rocky {
        0.24 + 0.76 * n.dot(light).max(0.0)
    } else {
        0.52 + 0.48 * n.dot(light).max(0.0)
    };
    let half = light.add(view).normalize();
    let spec = n.dot(half).max(0.0).powi(if rocky {
        12
    } else if gaseous {
        22
    } else {
        90
    });
    let fresnel = (1.0 - n.dot(view).max(0.0)).powi(3);
    let c = base.map(|v| v * diffuse);
    let c = mix(
        c,
        mix(MINT, PINK, 0.5 + 0.5 * (p.y * 4.0).sin()),
        fresnel
            * if rocky {
                0.0
            } else if gaseous {
                0.08
            } else {
                0.6
            },
    );
    let strength = if rocky {
        0.035
    } else if gaseous {
        0.12
    } else {
        0.92
    };
    let c = mix(
        c,
        [1.0, 1.0, 1.0],
        (spec * strength
            + fresnel
                * if rocky {
                    0.0
                } else if gaseous {
                    0.02
                } else {
                    0.22
                })
        .min(1.0),
    );
    let c = c.map(|v| (v.clamp(0.0, 1.0) * 255.0).round() as u32);
    Some(c[0] << 16 | c[1] << 8 | c[2])
}

pub fn is_glass(design: usize, part: u8) -> bool {
    matches!(design, 0 | 3 | 4 | 5) && (part == 0 || design == 4 && matches!(part, 6 | 8 | 9))
}

/// Rotation columns map object-local directions into camera/world coordinates.
pub fn material(
    design: usize,
    part: u8,
    p: Vec3,
    mut n: Vec3,
    world: Vec3,
    basis: [Vec3; 3],
) -> Option<(u32, f64)> {
    if design == 1 {
        // Finite-difference bump mapping changes lighting, not mesh positions.
        let q = p.scale(18.0);
        let e = 0.025;
        let value = noise(q);
        let gradient = Vec3::new(
            noise(q.add(Vec3::new(e, 0.0, 0.0))) - value,
            noise(q.add(Vec3::new(0.0, e, 0.0))) - value,
            noise(q.add(Vec3::new(0.0, 0.0, e))) - value,
        )
        .scale(1.0 / e);
        let g = basis[0]
            .scale(gradient.x)
            .add(basis[1].scale(gradient.y))
            .add(basis[2].scale(gradient.z));
        let tangent = g.sub(n.scale(g.dot(n)));
        n = n.sub(tangent.scale(0.25)).normalize();
    }
    let color = shade(design, part, p, n, world)?;
    if !is_glass(design, part) {
        return Some((color, 1.0));
    }
    let view = Vec3::new(0.0, 0.0, 5.0).sub(world).normalize();
    let local_view = Vec3::new(view.dot(basis[0]), view.dot(basis[1]), view.dot(basis[2]));
    let facing = n.normalize().dot(view).abs();
    let fresnel = (1.0 - facing).powi(3);
    let mut inside = [0.0; 3];
    let mut density = 0.0;
    // A short local ray samples nested filaments beneath the surface. This is
    // an artistic volume approximation; it does not trace refracted rays.
    let thickness = match design {
        0 => 0.65,
        3 => 0.22,
        4 => 0.18,
        _ => 0.10,
    };
    for i in 0..5 {
        let q = p.sub(local_view.scale(thickness * (i as f64 + 1.0) / 5.0));
        let field = noise(q.scale(7.0));
        let wave = match design {
            0 => (q.x * 18.0 + q.y * 13.0 + field * 5.0).sin(),
            3 => (q.x * 12.0 + q.y * 17.0 + field * 4.0).sin(),
            4 => (q.y * 17.0 + (q.x * 5.0 + q.z * 3.0).sin() * 2.3 + field * 2.0).sin(),
            _ => (q.y.atan2(q.x) * 5.0 - (q.x * q.x + q.y * q.y).sqrt() * 13.0 + q.z * 4.0 + field)
                .sin(),
        };
        let d = (1.0 - smooth(0.08, 0.35, wave.abs())) * (1.0 - i as f64 * 0.1);
        let tint = mix(MINT, PINK, field);
        for j in 0..3 {
            inside[j] += tint[j] * d;
        }
        density += d;
    }
    let surface = [
        ((color >> 16) & 255) as f64 / 255.0,
        ((color >> 8) & 255) as f64 / 255.0,
        (color & 255) as f64 / 255.0,
    ];
    let c = if density > 0.001 {
        mix(surface, inside.map(|v| v / density), 0.38)
    } else {
        surface
    };
    let c = c.map(|v| ((v * 1.12).clamp(0.0, 1.0) * 255.0).round() as u32);
    let alpha = (0.40 + 0.40 * fresnel + 0.10 * (density / 5.0)).clamp(0.40, 0.90);
    Some((c[0] << 16 | c[1] << 8 | c[2], alpha))
}

pub fn interpolate(values: [Vec3; 3], w: [f64; 3]) -> Vec3 {
    values[0]
        .scale(w[0])
        .add(values[1].scale(w[1]))
        .add(values[2].scale(w[2]))
}

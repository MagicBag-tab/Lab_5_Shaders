use crate::{
    shaders::{interpolate, mix, noise, smooth},
    triangle::Vec3,
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Mode {
    Crystal,
    Flat,
    Gouraud,
    Phong,
    Toon,
    Terrain,
    Dissolve,
}
impl Mode {
    pub const ALL: [Self; 7] = [
        Self::Crystal,
        Self::Flat,
        Self::Gouraud,
        Self::Phong,
        Self::Toon,
        Self::Terrain,
        Self::Dissolve,
    ];
    pub fn name(self) -> &'static str {
        match self {
            Self::Crystal => "CRISTAL",
            Self::Flat => "FLAT",
            Self::Gouraud => "GOURAUD",
            Self::Phong => "PHONG",
            Self::Toon => "TOON",
            Self::Terrain => "TERRENO FBM",
            Self::Dissolve => "DISCARD",
        }
    }
}
type Color = [f64; 3];
fn pack(c: Color) -> u32 {
    let c = c.map(|v| (v.clamp(0.0, 1.0) * 255.0).round() as u32);
    c[0] << 16 | c[1] << 8 | c[2]
}
fn lighting(n: Vec3, p: Vec3, base: Color) -> Color {
    let view = Vec3::new(0.0, 0.0, 5.0).sub(p).normalize();
    let mut n = n.normalize();
    if n.dot(view) < 0.0 {
        n = n.scale(-1.0);
    }
    let light = Vec3::new(-0.5, 0.7, 1.0).normalize();
    let diffuse = n.dot(light).max(0.0);
    // Phong reflection-vector specular, rather than Blinn's half vector.
    let reflected = n.scale(2.0 * n.dot(light)).sub(light);
    let specular = if diffuse > 0.0 {
        reflected.dot(view).max(0.0).powi(48)
    } else {
        0.0
    };
    base.map(|v| v * (0.2 + 0.8 * diffuse) + 0.8 * specular)
}
pub fn fbm(mut p: Vec3) -> f64 {
    let mut value = 0.0;
    let mut amplitude = 0.5333333333;
    for _ in 0..4 {
        value += amplitude * noise(p);
        p = p.scale(2.03).add(Vec3::new(7.1, 3.7, 1.9));
        amplitude *= 0.5;
    }
    value
}

pub struct FaceShader {
    mode: Mode,
    world: [Vec3; 3],
    normals: [Vec3; 3],
    local: [Vec3; 3],
    flat: Color,
    gouraud: [Color; 3],
    threshold: f64,
}
impl FaceShader {
    pub fn new(
        mode: Mode,
        world: [Vec3; 3],
        normals: [Vec3; 3],
        local: [Vec3; 3],
        threshold: f64,
    ) -> Self {
        let base = [0.64, 0.45, 0.83];
        let center = interpolate(world, [1.0 / 3.0; 3]);
        let normal = world[1]
            .sub(world[0])
            .cross(world[2].sub(world[0]))
            .normalize();
        let flat = if mode == Mode::Flat {
            lighting(normal, center, base)
        } else {
            [0.0; 3]
        };
        let gouraud = if mode == Mode::Gouraud {
            std::array::from_fn(|i| lighting(normals[i], world[i], base))
        } else {
            [[0.0; 3]; 3]
        };
        Self {
            mode,
            world,
            normals,
            local,
            flat,
            gouraud,
            threshold,
        }
    }
    pub fn shade(&self, w: [f64; 3]) -> Option<u32> {
        if self.mode == Mode::Flat {
            return Some(pack(self.flat));
        }
        if self.mode == Mode::Gouraud {
            return Some(pack(std::array::from_fn(|channel| {
                (0..3).map(|i| self.gouraud[i][channel] * w[i]).sum()
            })));
        }
        let p = interpolate(self.world, w);
        let n = interpolate(self.normals, w).normalize();
        let local = interpolate(self.local, w);
        let color = match self.mode {
            Mode::Toon => {
                let view = Vec3::new(0.0, 0.0, 5.0).sub(p).normalize();
                let n = if n.dot(view) < 0.0 { n.scale(-1.0) } else { n };
                let d = n.dot(Vec3::new(-0.5, 0.7, 1.0).normalize()).max(0.0);
                let band = if d < 0.25 {
                    0.25
                } else if d < 0.6 {
                    0.55
                } else {
                    1.0
                };
                let rim = 1.0 - smooth(0.0, 0.22, n.dot(view));
                mix(
                    [0.75 * band, 0.5 * band, 0.9 * band],
                    [0.13, 0.06, 0.22],
                    rim,
                )
            }
            Mode::Terrain => {
                let h = fbm(local.scale(3.0));
                let c = mix([0.12, 0.24, 0.65], [0.3, 0.76, 0.83], smooth(0.3, 0.48, h));
                let c = mix(c, [0.96, 0.84, 0.62], smooth(0.47, 0.51, h));
                let c = mix(c, [0.3, 0.65, 0.43], smooth(0.51, 0.56, h));
                let c = mix(c, [0.96, 0.94, 1.0], smooth(0.68, 0.8, h));
                lighting(n, p, c)
            }
            Mode::Dissolve => {
                let value = fbm(local.scale(6.0));
                if value < self.threshold {
                    return None;
                }
                let edge = 1.0 - smooth(self.threshold, self.threshold + 0.045, value);
                mix(lighting(n, p, [0.64, 0.45, 0.83]), [1.0, 0.73, 0.23], edge)
            }
            _ => lighting(n, p, [0.64, 0.45, 0.83]),
        };
        Some(pack(color))
    }
}

pub const POST_NAMES: [&str; 4] = ["NORMAL", "GRISES", "INVERSION", "BORDES"];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::triangle::{Triangle, raster_shader};

    #[test]
    fn flat_is_constant_and_phong_differs_from_vertex_color_interpolation() {
        let world = [
            Vec3::new(-1.0, -1.0, 0.0),
            Vec3::new(1.0, -1.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
        ];
        let normals = world.map(|v| v.add(Vec3::new(0.0, 0.0, 1.0)).normalize());
        let flat = FaceShader::new(Mode::Flat, world, normals, world, 0.5);
        assert_eq!(flat.shade([1.0, 0.0, 0.0]), flat.shade([0.0, 0.5, 0.5]));
        let gouraud = FaceShader::new(Mode::Gouraud, world, normals, world, 0.5);
        let phong = FaceShader::new(Mode::Phong, world, normals, world, 0.5);
        assert_ne!(gouraud.shade([0.3, 0.3, 0.4]), phong.shade([0.3, 0.3, 0.4]));
    }

    #[test]
    fn discard_writes_neither_color_nor_depth() {
        let points = [
            Vec3::new(-1.0, -1.0, 0.0),
            Vec3::new(1.0, -1.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
        ];
        let t = Triangle::new(points[0], points[1], points[2], 0);
        let shader = FaceShader::new(Mode::Dissolve, points, t.normals, points, 1.01);
        let mut color = vec![0x123456; 32 * 32];
        let mut depth = vec![0.0; 32 * 32];
        raster_shader(&mut color, &mut depth, 32, 32, &t, 10.0, 16.0, 16.0, |w| {
            shader.shade(w)
        });
        assert!(color.iter().all(|&c| c == 0x123456));
        assert!(depth.iter().all(|&d| d == 0.0));
        let shader = FaceShader::new(Mode::Dissolve, points, t.normals, points, 0.0);
        raster_shader(&mut color, &mut depth, 32, 32, &t, 10.0, 16.0, 16.0, |w| {
            shader.shade(w)
        });
        assert!(depth.iter().any(|&d| d > 0.0));
    }

    #[test]
    fn postprocessing_and_noise_are_deterministic() {
        let p = Vec3::new(-3.2, 1.7, 0.1);
        assert_eq!(fbm(p), fbm(p));
        assert!((0.0..=1.0).contains(&fbm(p)));
        let mut scratch = Vec::new();
        let original = vec![0xFF0000, 0x00FF00, 0x0000FF, 0xABCDEF];
        let mut pixels = original.clone();
        postprocess(&mut pixels, &mut scratch, 2, 2);
        postprocess(&mut pixels, &mut scratch, 2, 2);
        assert_eq!(pixels, original);
        postprocess(&mut pixels, &mut scratch, 2, 1);
        assert!(
            pixels
                .iter()
                .all(|&p| p & 255 == (p >> 8) & 255 && p & 255 == (p >> 16) & 255)
        );
        let mut pixels = vec![0x554433; 25];
        postprocess(&mut pixels, &mut scratch, 5, 3);
        assert!(pixels.iter().all(|&p| p == 0));
    }
}
pub fn postprocess(buffer: &mut [u32], source: &mut Vec<u32>, width: usize, mode: usize) {
    if mode == 0 {
        return;
    }
    let gray = |c: u32| {
        (((c >> 16) & 255) as f64 * 0.2126
            + ((c >> 8) & 255) as f64 * 0.7152
            + (c & 255) as f64 * 0.0722) as u32
    };
    if mode == 1 {
        for c in buffer {
            let g = gray(*c);
            *c = g * 0x010101;
        }
    } else if mode == 2 {
        for c in buffer {
            *c ^= 0xFFFFFF;
        }
    } else {
        source.clear();
        source.extend_from_slice(buffer);
        buffer.fill(0);
        let height = source.len() / width;
        for y in 1..height - 1 {
            for x in 1..width - 1 {
                let at = |dx: isize, dy: isize| {
                    gray(source[(y as isize + dy) as usize * width + (x as isize + dx) as usize])
                        as f64
                };
                let gx = -at(-1, -1) + at(1, -1) - 2.0 * at(-1, 0) + 2.0 * at(1, 0) - at(-1, 1)
                    + at(1, 1);
                let gy = -at(-1, -1) - 2.0 * at(0, -1) - at(1, -1)
                    + at(-1, 1)
                    + 2.0 * at(0, 1)
                    + at(1, 1);
                let g = (gx * gx + gy * gy).sqrt().min(255.0) as u32;
                buffer[y * width + x] = g * 0x010101;
            }
        }
    }
}

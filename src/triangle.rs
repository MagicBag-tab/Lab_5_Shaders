use std::{
    collections::{HashMap, HashSet},
    fs,
    path::Path,
};
#[cfg(test)]
#[path = "render_tests.rs"]
mod tests;
pub const CAMERA_Z: f64 = 5.0;
pub const NEAR: f64 = 0.1;
pub const BASE_COLOR: u32 = 0x87CEEB;

pub struct Mesh {
    pub vertices: Vec<Vec3>,
    pub faces: Vec<(usize, usize, usize)>,
    pub face_materials: Vec<Option<String>>,
    pub materials: HashMap<String, u32>,
}
impl Mesh {
    pub fn from_obj(path: &str) -> Result<Self, String> {
        let text = fs::read_to_string(path).map_err(|e| e.to_string())?;
        let mut mesh = Self {
            vertices: vec![],
            faces: vec![],
            face_materials: vec![],
            materials: HashMap::new(),
        };
        let mut material = None;
        let mut seen = HashSet::new();
        for line in text.lines() {
            let mut tokens = line.split('#').next().unwrap_or("").split_whitespace();
            match tokens.next() {
                Some("v") => {
                    let v = tokens
                        .take(3)
                        .map(str::parse::<f64>)
                        .collect::<Result<Vec<_>, _>>()
                        .map_err(|e| e.to_string())?;
                    if v.len() != 3 || v.iter().any(|x| !x.is_finite()) {
                        return Err("Vertice invalido".into());
                    }
                    mesh.vertices.push(Vec3::new(v[0], v[1], v[2]));
                }
                Some("mtllib") => {
                    for name in tokens {
                        let file = Path::new(path)
                            .parent()
                            .unwrap_or(Path::new("."))
                            .join(name);
                        let mtl = fs::read_to_string(&file)
                            .map_err(|e| format!("{}: {e}", file.display()))?;
                        let mut name = String::new();
                        for line in mtl.lines() {
                            let mut parts = line.split('#').next().unwrap_or("").split_whitespace();
                            match parts.next() {
                                Some("newmtl") => name = parts.collect::<Vec<_>>().join(" "),
                                Some("Kd") => {
                                    let rgb = parts
                                        .take(3)
                                        .map(str::parse::<f64>)
                                        .collect::<Result<Vec<_>, _>>()
                                        .map_err(|e| e.to_string())?;
                                    if rgb.len() != 3 || rgb.iter().any(|x| !x.is_finite()) {
                                        return Err("Kd invalido".into());
                                    }
                                    let c =
                                        |i: usize| (rgb[i].clamp(0.0, 1.0) * 255.0).round() as u32;
                                    mesh.materials
                                        .insert(name.clone(), c(0) << 16 | c(1) << 8 | c(2));
                                }
                                _ => {}
                            }
                        }
                    }
                }
                Some("usemtl") => material = Some(tokens.collect::<Vec<_>>().join(" ")),
                Some("f") => {
                    let ids = tokens
                        .map(|t| {
                            let n = t
                                .split('/')
                                .next()
                                .unwrap_or("")
                                .parse::<isize>()
                                .map_err(|e| e.to_string())?;
                            let i = if n < 0 {
                                mesh.vertices.len() as isize + n
                            } else {
                                n - 1
                            };
                            if n == 0 || i < 0 || i as usize >= mesh.vertices.len() {
                                return Err("Indice fuera de rango".to_string());
                            }
                            Ok(i as usize)
                        })
                        .collect::<Result<Vec<_>, String>>()?;
                    // El modelo ya esta triangulado: no crear abanicos ambiguos para caras concavas.
                    if ids.len() != 3 {
                        return Err("El OBJ debe estar triangulado".into());
                    }
                    let mut key = ids
                        .iter()
                        .map(|&i| {
                            let v = mesh.vertices[i];
                            [v.x.to_bits(), v.y.to_bits(), v.z.to_bits()]
                        })
                        .collect::<Vec<_>>();
                    key.sort();
                    if seen.insert((key, material.clone())) {
                        mesh.faces.push((ids[0], ids[1], ids[2]));
                        mesh.face_materials.push(material.clone());
                    }
                }
                _ => {}
            }
        }
        Ok(mesh)
    }
    pub fn get_triangles(&self) -> Vec<Triangle> {
        self.faces
            .iter()
            .enumerate()
            .map(|(i, &(a, b, c))| {
                Triangle::new(
                    self.vertices[a],
                    self.vertices[b],
                    self.vertices[c],
                    self.face_materials[i]
                        .as_ref()
                        .and_then(|n| self.materials.get(n))
                        .copied()
                        .unwrap_or(BASE_COLOR),
                )
            })
            .collect()
    }
    pub fn calcular_centro(&self) -> Vec3 {
        let sum = self
            .vertices
            .iter()
            .fold(Vec3::new(0.0, 0.0, 0.0), |a, &b| a.add(b));
        scale(sum, 1.0 / self.vertices.len().max(1) as f64)
    }
    pub fn calcular_radio_max(&self, center: Vec3) -> f64 {
        self.vertices
            .iter()
            .map(|v| v.sub(center).length())
            .fold(0.0, f64::max)
    }
}
fn scale(v: Vec3, s: f64) -> Vec3 {
    Vec3::new(v.x * s, v.y * s, v.z * s)
}
fn recortar(t: &Triangle) -> Vec<Vec3> {
    let input = [t.v1, t.v2, t.v3];
    if input
        .iter()
        .any(|v| !v.x.is_finite() || !v.y.is_finite() || !v.z.is_finite())
    {
        return vec![];
    }
    let mut out = vec![];
    for i in 0..3 {
        let a = input[i];
        let b = input[(i + 1) % 3];
        let ia = CAMERA_Z - a.z >= NEAR;
        let ib = CAMERA_Z - b.z >= NEAR;
        if ia {
            out.push(a);
        }
        if ia != ib {
            let mut v = a.add(scale(b.sub(a), (CAMERA_Z - NEAR - a.z) / (b.z - a.z)));
            v.z = CAMERA_Z - NEAR;
            out.push(v);
        }
    }
    out
}
fn proyectar(v: Vec3, s: f64, ox: f64, oy: f64) -> (f64, f64, f64) {
    let q = 1.0 / (CAMERA_Z - v.z);
    (v.x * CAMERA_Z * q * s + ox, -v.y * CAMERA_Z * q * s + oy, q)
}
pub fn dibujar_triangulo(buffer: &mut [u32], width: usize, t: &Triangle, s: f64, ox: f64, oy: f64) {
    if width == 0 {
        return;
    }
    let height = buffer.len() / width;
    let polygon = recortar(t);
    for i in 0..polygon.len() {
        let a = proyectar(polygon[i], s, ox, oy);
        let b = proyectar(polygon[(i + 1) % polygon.len()], s, ox, oy);
        if ![a.0, a.1, b.0, b.1].iter().all(|v| v.is_finite()) {
            continue;
        }
        let dx = b.0 - a.0;
        let dy = b.1 - a.1;
        let mut lo: f64 = 0.0;
        let mut hi: f64 = 1.0;
        // Liang-Barsky limita el recorrido al area visible.
        for (p, q) in [
            (-dx, a.0),
            (dx, width as f64 - 1.0 - a.0),
            (-dy, a.1),
            (dy, height as f64 - 1.0 - a.1),
        ] {
            if p == 0.0 {
                if q < 0.0 {
                    hi = -1.0;
                }
            } else if p < 0.0 {
                lo = lo.max(q / p);
            } else {
                hi = hi.min(q / p);
            }
        }
        if lo > hi {
            continue;
        }
        let steps = (dx * (hi - lo)).abs().max((dy * (hi - lo)).abs()).ceil() as usize;
        for step in 0..=steps {
            let u = lo + (hi - lo) * step as f64 / steps.max(1) as f64;
            let x = (a.0 + dx * u).round() as isize;
            let y = (a.1 + dy * u).round() as isize;
            if x >= 0 && y >= 0 && (x as usize) < width && (y as usize) < height {
                buffer[y as usize * width + x as usize] = t.color;
            }
        }
    }
}
pub fn dibujar_triangulo_relleno(
    buffer: &mut [u32],
    depth: &mut [f64],
    width: usize,
    height: usize,
    t: &Triangle,
    s: f64,
    ox: f64,
    oy: f64,
) {
    if width == 0
        || height == 0
        || width
            .checked_mul(height)
            .is_none_or(|n| n > buffer.len() || n > depth.len())
    {
        return;
    }
    let polygon = recortar(t);
    for i in 1..polygon.len().saturating_sub(1) {
        let p = [polygon[0], polygon[i], polygon[i + 1]].map(|v| proyectar(v, s, ox, oy));
        if p.iter()
            .any(|v| !v.0.is_finite() || !v.1.is_finite() || !v.2.is_finite())
        {
            continue;
        }
        let edge = |a: (f64, f64, f64), b: (f64, f64, f64), x: f64, y: f64| {
            (b.0 - a.0) * (y - a.1) - (b.1 - a.1) * (x - a.0)
        };
        let area = edge(p[0], p[1], p[2].0, p[2].1);
        if !area.is_finite() || area.abs() < 1e-10 {
            continue;
        }
        let minx = p
            .iter()
            .map(|v| v.0)
            .fold(f64::INFINITY, f64::min)
            .floor()
            .max(0.0);
        let maxx = p
            .iter()
            .map(|v| v.0)
            .fold(f64::NEG_INFINITY, f64::max)
            .ceil()
            .min(width as f64 - 1.0);
        let miny = p
            .iter()
            .map(|v| v.1)
            .fold(f64::INFINITY, f64::min)
            .floor()
            .max(0.0);
        let maxy = p
            .iter()
            .map(|v| v.1)
            .fold(f64::NEG_INFINITY, f64::max)
            .ceil()
            .min(height as f64 - 1.0);
        if minx > maxx || miny > maxy {
            continue;
        }
        for y in miny as usize..=maxy as usize {
            for x in minx as usize..=maxx as usize {
                let w = [
                    edge(p[1], p[2], x as f64 + 0.5, y as f64 + 0.5) / area,
                    edge(p[2], p[0], x as f64 + 0.5, y as f64 + 0.5) / area,
                    edge(p[0], p[1], x as f64 + 0.5, y as f64 + 0.5) / area,
                ];
                if w.iter().any(|v| *v < 0.0) {
                    continue;
                }
                // 1/(5-z) es afin en pantalla; interpolar z directamente seria incorrecto.
                let q = w[0] * p[0].2 + w[1] * p[1].2 + w[2] * p[2].2;
                let index = y * width + x;
                if q > depth[index] {
                    depth[index] = q;
                    buffer[index] = t.color;
                }
            }
        }
    }
}
pub fn iluminar(color: u32, a: Vec3, b: Vec3, c: Vec3) -> u32 {
    let n = b.sub(a).cross(c.sub(a)).normalize();
    let intensity = 0.3 + 0.7 * n.dot(Vec3::new(0.3, 0.5, 1.0).normalize()).abs();
    let channel =
        |shift: u32| ((((color >> shift) & 255u32) as f64 * intensity).round() as u32) << shift;
    channel(16) | channel(8) | channel(0)
}

pub const WINDOW_WIDTH: usize = 800;
pub const WINDOW_HEIGHT: usize = 600;

#[derive(Clone, Copy, Debug)]
pub struct Vec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vec3 {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Vec3 { x, y, z }
    }
    pub fn sub(self, otro: Vec3) -> Vec3 {
        Vec3 {
            x: self.x - otro.x,
            y: self.y - otro.y,
            z: self.z - otro.z,
        }
    }
    pub fn add(self, otro: Vec3) -> Vec3 {
        Vec3 {
            x: self.x + otro.x,
            y: self.y + otro.y,
            z: self.z + otro.z,
        }
    }
    pub fn dot(self, otro: Vec3) -> f64 {
        self.x * otro.x + self.y * otro.y + self.z * otro.z
    }
    pub fn cross(self, otro: Vec3) -> Vec3 {
        Vec3 {
            x: self.y * otro.z - self.z * otro.y,
            y: self.z * otro.x - self.x * otro.z,
            z: self.x * otro.y - self.y * otro.x,
        }
    }
    pub fn length(self) -> f64 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }
    pub fn normalize(self) -> Vec3 {
        let len = self.length();
        if len == 0.0 {
            return Vec3::new(0.0, 0.0, 0.0);
        }
        Vec3 {
            x: self.x / len,
            y: self.y / len,
            z: self.z / len,
        }
    }
}

#[derive(Clone, Copy)]
pub struct Triangle {
    pub v1: Vec3,
    pub v2: Vec3,
    pub v3: Vec3,
    pub color: u32,
}

impl Triangle {
    pub fn new(v1: Vec3, v2: Vec3, v3: Vec3, color: u32) -> Self {
        Triangle { v1, v2, v3, color }
    }
}

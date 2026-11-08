//! Generador determinista de la geometría de los cuerpos celestes del laboratorio.
//! Ejecutar desde la raíz con: cargo run --bin generate_models

use std::{f64::consts::PI, fs, path::Path};

#[derive(Clone, Copy)]
struct P {
    x: f64,
    y: f64,
    z: f64,
}
impl P {
    fn n(self) -> P {
        let l = (self.x * self.x + self.y * self.y + self.z * self.z).sqrt();
        P {
            x: self.x / l,
            y: self.y / l,
            z: self.z / l,
        }
    }
}
struct Obj {
    v: Vec<P>,
    f: Vec<[usize; 3]>,
}
impl Obj {
    fn new() -> Self {
        Self {
            v: vec![],
            f: vec![],
        }
    }
    fn tri(&mut self, a: P, b: P, c: P) {
        let i = self.v.len();
        self.v.extend([a, b, c]);
        self.f.push([i, i + 1, i + 2]);
    }
    fn save(&self, path: &str) {
        let mut s = String::from("# Generated procedurally; no materials or textures.\n");
        for p in &self.v {
            s += &format!("v {:.6} {:.6} {:.6}\n", p.x, p.y, p.z);
        }
        for p in &self.v {
            let n = p.n();
            s += &format!("vn {:.6} {:.6} {:.6}\n", n.x, n.y, n.z);
        }
        for [a, b, c] in &self.f {
            s += &format!(
                "f {}//{} {}//{} {}//{}\n",
                a + 1,
                a + 1,
                b + 1,
                b + 1,
                c + 1,
                c + 1
            );
        }
        fs::write(path, s).unwrap();
    }
}
fn sphere(name: &str, r: f64, nu: usize, nv: usize) -> Obj {
    let mut o = Obj::new();
    for j in 0..nv {
        let a = PI * j as f64 / nv as f64;
        let b = PI * (j + 1) as f64 / nv as f64;
        for i in 0..nu {
            let u = 2. * PI * i as f64 / nu as f64;
            let v = 2. * PI * (i + 1) as f64 / nu as f64;
            let p = |a: f64, u: f64| P {
                x: r * a.sin() * u.cos(),
                y: r * a.cos(),
                z: r * a.sin() * u.sin(),
            };
            let q = [p(a, u), p(a, v), p(b, v), p(b, u)];
            o.tri(q[0], q[1], q[2]);
            o.tri(q[0], q[2], q[3]);
        }
    }
    o.save(&format!("assets/{name}.obj"));
    o
}
fn torus(name: &str, major: f64, minor: f64, tilt: f64) -> Obj {
    let mut o = Obj::new();
    let nu = 48;
    let nv = 10;
    for i in 0..nu {
        for j in 0..nv {
            let a = 2. * PI * i as f64 / nu as f64;
            let b = 2. * PI * j as f64 / nv as f64;
            let c = 2. * PI * (i + 1) as f64 / nu as f64;
            let d = 2. * PI * (j + 1) as f64 / nv as f64;
            let p = |x: f64, y: f64| {
                let q = P {
                    x: (major + minor * y.cos()) * x.cos(),
                    y: minor * y.sin(),
                    z: (major + minor * y.cos()) * x.sin(),
                };
                P {
                    x: q.x,
                    y: q.y * tilt.cos() - q.z * tilt.sin(),
                    z: q.y * tilt.sin() + q.z * tilt.cos(),
                }
            };
            let q = [p(a, b), p(c, b), p(c, d), p(a, d)];
            o.tri(q[0], q[1], q[2]);
            o.tri(q[0], q[2], q[3]);
        }
    }
    o.save(&format!("assets/{name}.obj"));
    o
}
fn corona() -> Obj {
    let mut o = Obj::new();
    let n = 10;
    for i in 0..n {
        let a = 2. * PI * i as f64 / n as f64;
        let b = 2. * PI * (i + 1) as f64 / n as f64;
        let r = 1.55;
        let tip = 2.05;
        let p = |t: f64, z: f64, r: f64| P {
            x: r * t.cos(),
            y: r * t.sin(),
            z,
        };
        o.tri(p(a, 0.0, r), p((a + b) / 2.0, 0.12, tip), p(b, 0.0, r));
        o.tri(p(a, 0.0, -r), p(b, 0.0, -r), p((a + b) / 2.0, -0.12, tip));
    }
    o.save("assets/solaria_corona.obj");
    o
}
fn crescent() -> Obj {
    let mut o = Obj::new();
    let n = 32;
    let mut a = Vec::new();
    for i in 0..=n {
        let t = -PI / 2.0 + PI * i as f64 / n as f64;
        a.push(P {
            x: 0.95 * t.cos() - 0.28,
            y: 1.2 * t.sin(),
            z: 0.0,
        });
    }
    for i in (0..=n).rev() {
        let t = -PI / 2.0 + PI * i as f64 / n as f64;
        a.push(P {
            x: 0.55 * t.cos() + 0.18,
            y: 1.0 * t.sin(),
            z: 0.0,
        });
    }
    for side in [-0.22, 0.22] {
        for p in &a {
            o.v.push(P {
                x: p.x,
                y: p.y,
                z: side,
            });
        }
    }
    let m = a.len();
    for i in 0..m {
        let j = (i + 1) % m;
        o.tri(o.v[i], o.v[j], o.v[m + j]);
        o.tri(o.v[i], o.v[m + j], o.v[m + i]);
    }
    o.save("assets/labyrinthia_body.obj");
    o
}
fn eye() -> Obj {
    let mut o = Obj::new();
    let nu = 40;
    let nv = 12;
    for j in 0..nv {
        let y = -1.0 + 2.0 * j as f64 / nv as f64;
        let y2 = -1.0 + 2.0 * (j + 1) as f64 / nv as f64;
        for i in 0..nu {
            let a = 2. * PI * i as f64 / nu as f64;
            let b = 2. * PI * (i + 1) as f64 / nu as f64;
            let p = |y: f64, t: f64| {
                let w = (1.0 - y * y).max(0.0).sqrt();
                P {
                    x: 1.35 * w * t.cos(),
                    y: 0.72 * y,
                    z: 0.55 * w * t.sin(),
                }
            };
            let q = [p(y, a), p(y, b), p(y2, b), p(y2, a)];
            o.tri(q[0], q[1], q[2]);
            o.tri(q[0], q[2], q[3]);
        }
    }
    o.save("assets/ocularis_body.obj");
    o
}
fn shells() -> Obj {
    let mut o = Obj::new();
    for cx in [-0.62, 0.62] {
        let nu = 20;
        let nv = 10;
        for j in 0..nv {
            let a = PI * j as f64 / (2. * nv as f64);
            let b = PI * (j + 1) as f64 / (2. * nv as f64);
            for i in 0..nu {
                let u = 2. * PI * i as f64 / nu as f64;
                let v = 2. * PI * (i + 1) as f64 / nu as f64;
                let p = |aa: f64, uu: f64| P {
                    x: cx + 0.72 * aa.sin() * uu.cos(),
                    y: 0.72 * aa.cos(),
                    z: 0.72 * aa.sin() * uu.sin(),
                };
                let q = [p(a, u), p(a, v), p(b, v), p(b, u)];
                o.tri(q[0], q[1], q[2]);
                o.tri(q[0], q[2], q[3]);
            }
        }
    }
    o.save("assets/vortex_shells.obj");
    o
}
fn main() {
    fs::create_dir_all(Path::new("assets")).unwrap();
    sphere("solaria_body", 1.0, 32, 16);
    sphere("lunaria_body", 1.0, 32, 16);
    sphere("lunaria_moon", 0.28, 20, 10);
    sphere("aurelia_body", 1.0, 32, 16);
    sphere("vortex_core", 0.28, 20, 10);
    torus("aurelia_rings", 1.45, 0.12, 0.35);
    torus("vortex_ring", 1.35, 0.08, 0.8);
    corona();
    crescent();
    eye();
    shells();
    torus("ocularis_upper_arc", 0.88, 0.06, 0.0);
    torus("ocularis_lower_arc", 0.72, 0.05, 0.0);
    println!("Modelos generados en assets/");
}

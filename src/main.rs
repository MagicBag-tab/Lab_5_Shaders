mod triangle;

use minifb::{Key, Window, WindowOptions};
use std::time::Instant;
use triangle::*;

const DESIGNS: [&str; 6] = [
    "SOLARIA",
    "LUNARIA",
    "AURELIA",
    "OCULARIS",
    "LABYRINTHIA",
    "VORTEX",
];
const DESIGN_PARTS: [&[&str]; 6] = [
    &["assets/solaria.obj"],
    &["assets/lunaria.obj"],
    &["assets/aurelia.obj"],
    &["assets/ocularis.obj"],
    &["assets/labyrinthia.obj"],
    &["assets/vortex.obj"],
];

fn load_design(index: usize) -> Result<(Vec<Triangle>, Vec3, f64), String> {
    let mut triangles = Vec::new();
    let mut vertices = Vec::new();
    for path in DESIGN_PARTS[index] {
        let mesh = Mesh::from_obj(path)?;
        vertices.extend(mesh.vertices.iter().copied());
        triangles.extend(mesh.get_triangles().into_iter().map(|mut t| {
            t.color = 0xFFFFFF;
            t
        }));
    }
    if vertices.is_empty() {
        return Err("El diseño no contiene vértices".into());
    }
    let center = vertices
        .iter()
        .fold(Vec3::new(0.0, 0.0, 0.0), |sum, &v| sum.add(v));
    let center = center.scale(1.0 / vertices.len() as f64);
    let radius = vertices
        .iter()
        .map(|&v| v.sub(center).length())
        .fold(0.0, f64::max);
    Ok((triangles, center, radius))
}

fn main() {
    let mut window = Window::new(
        "Cuerpos celestes - SOLARIA [WIREframe]",
        WINDOW_WIDTH,
        WINDOW_HEIGHT,
        WindowOptions::default(),
    )
    .unwrap();
    let mut selected = 0usize;
    let mut loaded = load_design(selected).expect("No se pudieron cargar los OBJ");
    let mut angle_y = 0.0f64;
    let mut angle_x = 0.0f64;
    let mut automatic = true;
    let mut buffer = vec![0u32; WINDOW_WIDTH * WINDOW_HEIGHT];
    let start = Instant::now();
    let mut previous = 0.0;

    while window.is_open() && !window.is_key_down(Key::Escape) {
        for (key, index) in [
            (Key::Key1, 0),
            (Key::Key2, 1),
            (Key::Key3, 2),
            (Key::Key4, 3),
            (Key::Key5, 4),
            (Key::Key6, 5),
        ] {
            if window.is_key_pressed(key, minifb::KeyRepeat::No) {
                selected = index;
                loaded = load_design(selected).expect("No se pudo cargar el diseño");
            }
        }
        if window.is_key_pressed(Key::Space, minifb::KeyRepeat::No) {
            automatic = !automatic;
        }
        if automatic {
            let now = start.elapsed().as_secs_f64();
            angle_y += (now - previous).min(0.1) * 0.8;
            previous = now;
        } else {
            if window.is_key_down(Key::Left) {
                angle_y -= 0.03;
            }
            if window.is_key_down(Key::Right) {
                angle_y += 0.03;
            }
            if window.is_key_down(Key::Up) {
                angle_x -= 0.03;
            }
            if window.is_key_down(Key::Down) {
                angle_x += 0.03;
            }
        }
        window.set_title(&format!(
            "Cuerpos celestes - {} [WIRE] [{}]",
            DESIGNS[selected],
            if automatic { "AUTO" } else { "MANUAL" }
        ));
        buffer.fill(0x000000);
        let (triangles, center, radius) = &loaded;
        let scale = WINDOW_WIDTH as f64 * 0.38 / radius.max(0.001);
        let (cos_y, sin_y) = (angle_y.cos(), angle_y.sin());
        let (cos_x, sin_x) = (angle_x.cos(), angle_x.sin());
        for triangle in triangles {
            let transform = |v: Vec3| {
                let v = v.sub(*center);
                let y = Vec3::new(v.x * cos_y + v.z * sin_y, v.y, -v.x * sin_y + v.z * cos_y);
                Vec3::new(
                    y.x,
                    y.y * cos_x - y.z * sin_x,
                    y.y * sin_x + y.z * cos_x - 8.0,
                )
            };
            let rendered = Triangle::new(
                transform(triangle.v1),
                transform(triangle.v2),
                transform(triangle.v3),
                0xFFFFFF,
            );
            dibujar_triangulo(&mut buffer, WINDOW_WIDTH, &rendered, scale, 400.0, 300.0);
        }
        window
            .update_with_buffer(&buffer, WINDOW_WIDTH, WINDOW_HEIGHT)
            .unwrap();
    }
}

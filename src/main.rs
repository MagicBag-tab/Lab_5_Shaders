mod motion;
mod shaders;
mod techniques;
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
        let mesh = Mesh::geometry_from_obj(path)?;
        vertices.extend(mesh.vertices.iter().copied());
        triangles.extend(mesh.get_triangles().into_iter().map(|mut t| {
            t.color = 0xFFFFFF;
            t
        }));
    }
    if vertices.is_empty() {
        return Err("El diseño no contiene vértices".into());
    }
    // Accessories orbit the archive's shared origin; fit their entire trajectory.
    let center = Vec3::new(0.0, 0.0, 0.0);
    let radius = vertices
        .iter()
        .map(|&v| v.sub(center).length())
        .fold(0.0, f64::max)
        + 0.32;
    Ok((triangles, center, radius))
}

fn main() {
    if std::env::args().any(|arg| arg == "--capture" || arg == "--capture-techniques") {
        capture_previews();
        return;
    }
    let mut window = Window::new(
        "Cuerpos celestes - SOLARIA [SOLIDO]",
        WINDOW_WIDTH,
        WINDOW_HEIGHT,
        WindowOptions::default(),
    )
    .unwrap();
    let mut selected = 0usize;
    let mut loaded = load_design(selected).expect("No se pudieron cargar los OBJ");
    let mut angle_y = 0.0f64;
    let mut angle_x = 0.0f64;
    let mut automatic = false;
    let mut accessories = true;
    let mut paused = false;
    let mut wireframe = false;
    let mut shaders_enabled = true;
    let mut mode = techniques::Mode::Crystal;
    let mut post = 0;
    let mut post_source = Vec::new();
    let mut dissolve = 0.5f64;
    let mut buffer = vec![0u32; WINDOW_WIDTH * WINDOW_HEIGHT];
    let mut depth = vec![0.0; buffer.len()];
    let start = Instant::now();
    let mut previous = 0.0;
    let mut animation_time = 0.0;

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
            paused = !paused;
        }
        if window.is_key_pressed(Key::G, minifb::KeyRepeat::No) {
            automatic = !automatic;
        }
        if window.is_key_pressed(Key::A, minifb::KeyRepeat::No) {
            accessories = !accessories;
        }
        if window.is_key_pressed(Key::R, minifb::KeyRepeat::No) {
            angle_x = 0.0;
            angle_y = 0.0;
            automatic = false;
        }
        if window.is_key_pressed(Key::T, minifb::KeyRepeat::No) {
            wireframe = !wireframe;
        }
        if window.is_key_pressed(Key::S, minifb::KeyRepeat::No) {
            shaders_enabled = !shaders_enabled;
        }
        for (key, next) in [
            Key::F1,
            Key::F2,
            Key::F3,
            Key::F4,
            Key::F5,
            Key::F6,
            Key::F7,
        ]
        .into_iter()
        .zip(techniques::Mode::ALL)
        {
            if window.is_key_pressed(key, minifb::KeyRepeat::No) {
                mode = next;
                shaders_enabled = true;
                wireframe = false;
            }
        }
        if window.is_key_pressed(Key::P, minifb::KeyRepeat::No) {
            post = (post + 1) % 4;
        }
        let now = start.elapsed().as_secs_f64();
        let dt = (now - previous).min(0.1);
        previous = now;
        if window.is_key_down(Key::Minus) {
            dissolve = (dissolve - dt * 0.3).max(0.0);
        }
        if window.is_key_down(Key::Equal) {
            dissolve = (dissolve + dt * 0.3).min(1.01);
        }
        if accessories && !paused {
            animation_time += dt;
        }
        if automatic && !paused {
            angle_y += dt * 0.18;
        }
        {
            if window.is_key_down(Key::Left) {
                angle_y -= dt * 1.5;
            }
            if window.is_key_down(Key::Right) {
                angle_y += dt * 1.5;
            }
            if window.is_key_down(Key::Up) {
                angle_x -= dt * 1.5;
            }
            if window.is_key_down(Key::Down) {
                angle_x += dt * 1.5;
            }
        }
        window.set_title(&format!(
            "Cuerpos celestes - {} [{}] [giro:{} accesorios:{}] [{}] discard={:.2}",
            DESIGNS[selected],
            if wireframe {
                "WIRE"
            } else if shaders_enabled {
                mode.name()
            } else {
                "SOLIDO"
            },
            if automatic && !paused { "ON" } else { "OFF" },
            if accessories && !paused { "ON" } else { "OFF" },
            techniques::POST_NAMES[post],
            dissolve
        ));
        buffer.fill(if shaders_enabled { 0x190D29 } else { 0 });
        depth.fill(0.0);
        let (triangles, center, radius) = &loaded;
        let scale = fit_scale(*radius);
        let (cos_y, sin_y) = (angle_y.cos(), angle_y.sin());
        let (cos_x, sin_x) = (angle_x.cos(), angle_x.sin());
        let motions: [_; 10] =
            std::array::from_fn(|part| motion::Motion::at(selected, part as u8, animation_time));
        for triangle in triangles {
            let motion = motions[triangle.part as usize];
            let transform = |v: Vec3| {
                let v = v.sub(*center);
                let y = Vec3::new(v.x * cos_y + v.z * sin_y, v.y, -v.x * sin_y + v.z * cos_y);
                Vec3::new(
                    y.x,
                    y.y * cos_x - y.z * sin_x,
                    y.y * sin_x + y.z * cos_x - 8.0,
                )
            };
            let rotate = |v: Vec3| {
                let y = Vec3::new(v.x * cos_y + v.z * sin_y, v.y, -v.x * sin_y + v.z * cos_y);
                Vec3::new(y.x, y.y * cos_x - y.z * sin_x, y.y * sin_x + y.z * cos_x)
            };
            let rendered = Triangle::new(
                transform(motion.position(triangle.v1)),
                transform(motion.position(triangle.v2)),
                transform(motion.position(triangle.v3)),
                0xFFFFFF,
            );
            if wireframe {
                dibujar_triangulo(&mut buffer, WINDOW_WIDTH, &rendered, scale, 400.0, 300.0);
            } else if shaders_enabled {
                let local = [triangle.v1, triangle.v2, triangle.v3];
                let world = [rendered.v1, rendered.v2, rendered.v3];
                let normals = triangle.normals.map(|v| rotate(motion.normal(v)));
                let teaching = techniques::FaceShader::new(mode, world, normals, local, dissolve);
                raster_shader(
                    &mut buffer,
                    &mut depth,
                    WINDOW_WIDTH,
                    WINDOW_HEIGHT,
                    &rendered,
                    scale,
                    400.0,
                    300.0,
                    |w| {
                        if mode != techniques::Mode::Crystal {
                            return teaching.shade(w);
                        }
                        shaders::shade(
                            selected,
                            triangle.part,
                            shaders::interpolate(local, w),
                            shaders::interpolate(normals, w),
                            shaders::interpolate(world, w),
                        )
                    },
                );
            } else {
                dibujar_triangulo_relleno(
                    &mut buffer,
                    &mut depth,
                    WINDOW_WIDTH,
                    WINDOW_HEIGHT,
                    &rendered,
                    scale,
                    400.0,
                    300.0,
                );
            }
        }
        techniques::postprocess(&mut buffer, &mut post_source, WINDOW_WIDTH, post);
        window
            .update_with_buffer(&buffer, WINDOW_WIDTH, WINDOW_HEIGHT)
            .unwrap();
    }
}

// Headless captures use the same CPU rasterizer and fragment shaders as the viewer.
fn capture_previews() {
    let seconds = std::env::args()
        .find_map(|arg| arg.strip_prefix("--time=").map(str::to_owned))
        .map(|v| {
            v.parse::<f64>()
                .expect("--time debe ser un numero de segundos")
        })
        .unwrap_or(0.0);
    assert!(seconds.is_finite() && seconds >= 0.0);
    let out = std::path::Path::new("assets/previews");
    std::fs::create_dir_all(out).unwrap();
    for index in 0..DESIGNS.len() {
        let modes = if std::env::args().any(|arg| arg == "--capture-techniques") {
            techniques::Mode::ALL.to_vec()
        } else {
            vec![techniques::Mode::Crystal]
        };
        for mode in modes {
            let pixels = shader_preview_frame(index, false, mode, seconds);
            let size = 800 * 600 * 3;
            let mut bmp = vec![0u8; 54];
            bmp[..2].copy_from_slice(b"BM");
            bmp[2..6].copy_from_slice(&((54 + size) as u32).to_le_bytes());
            bmp[10..14].copy_from_slice(&54u32.to_le_bytes());
            bmp[14..18].copy_from_slice(&40u32.to_le_bytes());
            bmp[18..22].copy_from_slice(&800i32.to_le_bytes());
            bmp[22..26].copy_from_slice(&600i32.to_le_bytes());
            bmp[26..28].copy_from_slice(&1u16.to_le_bytes());
            bmp[28..30].copy_from_slice(&24u16.to_le_bytes());
            for row in pixels.chunks(800).rev() {
                for &pixel in row {
                    bmp.extend([pixel as u8, (pixel >> 8) as u8, (pixel >> 16) as u8]);
                }
            }
            let suffix = if mode == techniques::Mode::Crystal {
                String::new()
            } else {
                format!("_{:?}", mode).to_lowercase()
            };
            let path = out.join(format!("{}{suffix}.bmp", DESIGNS[index].to_lowercase()));
            std::fs::write(&path, bmp).unwrap();
            println!("{}", path.display());
        }
    }
}

fn fit_scale(radius: f64) -> f64 {
    // Fit a bounding sphere at distance 13, including its nearest surface.
    let radius = radius.max(0.001);
    WINDOW_HEIGHT as f64 * 0.43 * (CAMERA_Z + 8.0 - radius).max(0.1) / (CAMERA_Z * radius)
}

#[cfg(test)]
fn shader_preview(index: usize, reverse: bool) -> Vec<u32> {
    shader_preview_mode(index, reverse, techniques::Mode::Crystal)
}

#[cfg(test)]
fn shader_preview_mode(index: usize, reverse: bool, mode: techniques::Mode) -> Vec<u32> {
    shader_preview_frame(index, reverse, mode, 0.0)
}

fn shader_preview_frame(
    index: usize,
    reverse: bool,
    mode: techniques::Mode,
    seconds: f64,
) -> Vec<u32> {
    let (mut triangles, center, radius) = load_design(index).unwrap();
    if reverse {
        triangles.reverse();
    }
    let mut pixels = vec![0x190D29; 800 * 600];
    let mut depth = vec![0.0; pixels.len()];
    for t in triangles {
        let motion = motion::Motion::at(index, t.part, seconds);
        let local = [t.v1, t.v2, t.v3];
        let world = local.map(|v| {
            motion
                .position(v)
                .sub(center)
                .add(Vec3::new(0.0, 0.0, -8.0))
        });
        let normals = t.normals.map(|v| motion.normal(v));
        let transformed = Triangle::new(world[0], world[1], world[2], 0);
        let teaching = techniques::FaceShader::new(mode, world, normals, local, 0.5);
        raster_shader(
            &mut pixels,
            &mut depth,
            800,
            600,
            &transformed,
            fit_scale(radius),
            400.0,
            300.0,
            |w| {
                if mode != techniques::Mode::Crystal {
                    return teaching.shade(w);
                }
                shaders::shade(
                    index,
                    t.part,
                    shaders::interpolate(local, w),
                    shaders::interpolate(normals, w),
                    shaders::interpolate(world, w),
                )
            },
        );
    }
    pixels
}

#[cfg(test)]
mod viewer_tests {
    use super::*;

    #[test]
    fn accessory_motion_stays_in_frame_envelope() {
        for index in 0..DESIGNS.len() {
            let (triangles, center, radius) = load_design(index).unwrap();
            for seconds in [0.0, 2.0, 8.0, 17.0, 41.0] {
                for t in &triangles {
                    let motion = motion::Motion::at(index, t.part, seconds);
                    for v in [t.v1, t.v2, t.v3] {
                        assert!(
                            motion.position(v).sub(center).length() <= radius,
                            "{} at {seconds}",
                            DESIGNS[index]
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn procedural_previews_are_colored_and_order_independent() {
        for index in 0..DESIGNS.len() {
            let pixels = shader_preview(index, false);
            let reversed = shader_preview(index, true);
            // Shared triangle edges can differ by one quantization level.
            let differences = pixels.iter().zip(&reversed).filter(|(a, b)| a != b).count();
            assert!(differences < 30, "{}: {differences}", DESIGNS[index]);
            let colors = pixels.into_iter().collect::<std::collections::HashSet<_>>();
            assert!(colors.len() > 100, "{}", DESIGNS[index]);
            let mesh = Mesh::geometry_from_obj(DESIGN_PARTS[index][0]).unwrap();
            assert!(mesh.materials.is_empty());
            assert!(
                mesh.face_normals
                    .iter()
                    .all(|ids| ids.iter().all(Option::is_some))
            );
            assert!(mesh.get_triangles().iter().any(|t| t.part != 0));
        }
    }

    #[test]
    fn all_designs_load_and_render_solid() {
        for index in 0..DESIGNS.len() {
            let (triangles, center, radius) = load_design(index).unwrap();
            assert!(radius.is_finite() && radius > 0.0);
            let mut pixels = vec![0; 128 * 128];
            let mut depth = vec![0.0; pixels.len()];
            for t in triangles {
                let transform = |v: Vec3| v.sub(center).add(Vec3::new(0.0, 0.0, -8.0));
                let t = Triangle::new(transform(t.v1), transform(t.v2), transform(t.v3), 0xFFFFFF);
                dibujar_triangulo_relleno(
                    &mut pixels,
                    &mut depth,
                    128,
                    128,
                    &t,
                    48.0 / radius,
                    64.0,
                    64.0,
                );
            }
            assert!(pixels.contains(&0xFFFFFF), "{}", DESIGNS[index]);
            assert!(pixels.iter().all(|&p| p == 0 || p == 0xFFFFFF));
            assert!(depth.iter().all(|q| q.is_finite()));
        }
    }
}

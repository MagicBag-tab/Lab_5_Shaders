use super::*;

fn render(triangles: &[Triangle]) -> (Vec<u32>, Vec<f64>) {
    let mut pixels = vec![0; 64 * 64];
    let mut depth = vec![0.0; pixels.len()];
    for t in triangles {
        dibujar_triangulo_relleno(&mut pixels, &mut depth, 64, 64, t, 12.0, 32.0, 32.0);
    }
    (pixels, depth)
}

#[test]
fn perspective_depth_and_draw_order() {
    // Mismos vertices proyectados: (16,16), (48,16), (32,48).
    let at =
        |x: f64, y: f64, d: f64| Vec3::new((x - 32.0) * d / 60.0, -(y - 32.0) * d / 60.0, 5.0 - d);
    let sloped = Triangle::new(
        at(16.0, 16.0, 1.0),
        at(48.0, 16.0, 8.0),
        at(32.0, 48.0, 8.0),
        1,
    );
    let flat = Triangle::new(
        at(16.0, 16.0, 4.0),
        at(48.0, 16.0, 4.0),
        at(32.0, 48.0, 4.0),
        2,
    );
    let (pixels, depth) = render(&[sloped, flat]);
    assert_eq!(pixels, render(&[flat, sloped]).0);
    // A esta altura, interpolar z linealmente elegiria erroneamente la cara plana.
    assert_eq!(pixels[27 * 64 + 32], 1);
    let w2 = (27.5 - 16.0) / 32.0;
    let w1 = (32.5 - 16.0 - 16.0 * w2) / 32.0;
    let expected = (1.0 - w1 - w2) + (w1 + w2) / 8.0;
    assert!((depth[27 * 64 + 32] - expected).abs() < 1e-12);
    let reversed = Triangle::new(sloped.v3, sloped.v2, sloped.v1, sloped.color);
    assert_eq!(render(&[sloped]).0, render(&[reversed]).0);
}

#[test]
fn clipping_degenerate_and_invalid_buffers() {
    let a = Vec3::new(-1.0, -1.0, 0.0);
    assert!(
        render(&[Triangle::new(a, a, a, 1)])
            .0
            .iter()
            .all(|&p| p == 0)
    );
    let behind = Triangle::new(
        Vec3::new(-1.0, -1.0, 6.0),
        Vec3::new(1.0, -1.0, 6.0),
        Vec3::new(0.0, 1.0, 6.0),
        1,
    );
    assert!(render(&[behind]).0.iter().all(|&p| p == 0));
    let crossing = Triangle::new(a, Vec3::new(1.0, -1.0, 0.0), Vec3::new(0.0, 1.0, 5.5), 1);
    let (pixels, depth) = render(&[crossing]);
    assert!(pixels.contains(&1));
    assert!(
        depth
            .iter()
            .all(|q| q.is_finite() && *q <= 1.0 / NEAR + 1e-10)
    );
    let outside = Triangle::new(
        Vec3::new(-100.0, 0.0, 0.0),
        Vec3::new(-90.0, 1.0, 0.0),
        Vec3::new(-90.0, -1.0, 0.0),
        1,
    );
    assert!(render(&[outside]).0.iter().all(|&p| p == 0));
    dibujar_triangulo_relleno(&mut [], &mut [], 0, 0, &crossing, 1.0, 0.0, 0.0);
    dibujar_triangulo_relleno(&mut [0], &mut [], 64, 64, &crossing, 1.0, 0.0, 0.0);
    let mut wire = vec![0; 64 * 64];
    dibujar_triangulo(&mut wire, 64, &crossing, 12.0, 32.0, 32.0);
    assert!(wire.contains(&1));
}

#[test]
fn actual_model_materials_duplicates_and_rotations() {
    let mesh = Mesh::from_obj("assets/nave_1.obj").unwrap();
    assert_eq!(mesh.vertices.len(), 730);
    assert_eq!(mesh.faces.len(), 756);
    assert_eq!(mesh.materials["Material"], 0xCCCCCC);
    assert!(
        mesh.face_materials
            .iter()
            .all(|m| m.as_deref() == Some("Material"))
    );
    let center = mesh.calcular_centro();
    for (ax, ay) in [(0.0_f64, 0.0_f64), (0.4, 0.8), (1.2, 2.3), (2.8, 4.7)] {
        let transform = |v: Vec3| {
            let v = v.sub(center);
            let x = v.x * ay.cos() + v.z * ay.sin();
            let z = -v.x * ay.sin() + v.z * ay.cos();
            Vec3::new(
                x,
                v.y * ax.cos() - z * ax.sin(),
                v.y * ax.sin() + z * ax.cos() - 8.0,
            )
        };
        let mut tris = mesh
            .get_triangles()
            .iter()
            .map(|t| {
                let a = transform(t.v1);
                let b = transform(t.v2);
                let c = transform(t.v3);
                Triangle::new(a, b, c, iluminar(t.color, a, b, c))
            })
            .collect::<Vec<_>>();
        let (forward, depth) = render(&tris);
        assert!(forward.iter().any(|&p| p != 0));
        assert!(depth.iter().all(|q| q.is_finite()));
        tris.reverse();
        assert_eq!(forward, render(&tris).0, "rotacion {ax}, {ay}");
    }
}

#[test]
fn obj_material_assignment_negative_indices_and_errors() {
    let dir = std::env::temp_dir().join(format!("nave_obj_test_{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    fs::write(
        dir.join("test.mtl"),
        "newmtl red\nKd 1 0 0\nnewmtl blue\nKd 0 0 1\n",
    )
    .unwrap();
    let obj = dir.join("test.obj");
    let vertices = "v 0 0 0\nv 1 0 0\nv 0 1 0\n";
    fs::write(
        &obj,
        format!(
            "mtllib test.mtl\n{vertices}usemtl red\nf 1 2 3\nusemtl blue\nf -3 -2 -1 # comment\n"
        ),
    )
    .unwrap();
    let mesh = Mesh::from_obj(obj.to_str().unwrap()).unwrap();
    assert_eq!(
        mesh.get_triangles()
            .iter()
            .map(|t| t.color)
            .collect::<Vec<_>>(),
        vec![0xFF0000, 0x0000FF]
    );
    fs::write(&obj, format!("{vertices}f 1 2 3\n")).unwrap();
    assert_eq!(
        Mesh::from_obj(obj.to_str().unwrap())
            .unwrap()
            .get_triangles()[0]
            .color,
        BASE_COLOR
    );
    for face in ["f 0 2 3", "f 1 2 9", "f 1 2 3 1"] {
        fs::write(&obj, format!("{vertices}{face}\n")).unwrap();
        assert!(Mesh::from_obj(obj.to_str().unwrap()).is_err());
    }
    fs::remove_file(obj).unwrap();
    fs::remove_file(dir.join("test.mtl")).unwrap();
    fs::remove_dir(dir).unwrap();
}

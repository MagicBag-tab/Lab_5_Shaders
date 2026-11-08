//! Rigid accessory transforms in scene coordinates, evaluated once per frame.
use crate::triangle::Vec3;

#[derive(Clone, Copy)]
pub struct Motion {
    axis: u8,
    sin: f64,
    cos: f64,
    offset: Vec3,
}
impl Motion {
    pub fn at(design: usize, part: u8, seconds: f64) -> Self {
        let (axis, speed) = match (design, part) {
            (0, 1) => (2, 0.40), // corona, around its front-facing axis
            (1, 2) => (1, 0.80), // moon orbit around the shared scene origin
            (2, 3) => (1, 0.32), // ring precession
            (5, 3) => (1, -0.35),
            (5, 7) => (1, 1.1),
            _ => (1, 0.0),
        };
        let angle = seconds * speed;
        let mut offset = Vec3::new(0.0, 0.0, 0.0);
        if design == 4 && matches!(part, 6 | 8 | 9) {
            let phase = match part {
                8 => 2.1,
                9 => 4.2,
                _ => 0.0,
            };
            offset.y = 0.10 * ((seconds * 1.3 + phase).sin() - phase.sin());
            offset.x = 0.05 * ((seconds * 0.9 + phase).sin() - phase.sin());
            offset.z = 0.07 * ((seconds * 1.1 + phase).cos() - phase.cos());
        }
        if design == 5 && part == 7 {
            // The sphere's rotation alone is invisible in monochrome; translate its center too.
            offset.x = 0.09 * (seconds * 1.2).sin();
            offset.z = 0.05 * (seconds * 0.9).sin();
        }
        Self {
            axis,
            sin: angle.sin(),
            cos: angle.cos(),
            offset,
        }
    }
    pub fn normal(self, v: Vec3) -> Vec3 {
        if self.axis == 2 {
            Vec3::new(
                v.x * self.cos - v.y * self.sin,
                v.x * self.sin + v.y * self.cos,
                v.z,
            )
        } else {
            Vec3::new(
                v.x * self.cos + v.z * self.sin,
                v.y,
                -v.x * self.sin + v.z * self.cos,
            )
        }
    }
    pub fn position(self, v: Vec3) -> Vec3 {
        self.normal(v).add(self.offset)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn transforms_are_rigid_and_start_at_original_layout() {
        let a = Vec3::new(1.25, 0.74, 0.14);
        let b = Vec3::new(1.1, 0.8, 0.2);
        for (design, part) in [
            (0, 1),
            (1, 2),
            (2, 3),
            (4, 6),
            (4, 8),
            (4, 9),
            (5, 3),
            (5, 7),
        ] {
            assert!(Motion::at(design, part, 0.0).position(a).sub(a).length() < 1e-12);
            let m = Motion::at(design, part, 13.7);
            assert!((m.position(a).sub(m.position(b)).length() - a.sub(b).length()).abs() < 1e-12);
            assert!((m.normal(a).length() - a.length()).abs() < 1e-12);
        }
        let moved = Motion::at(1, 2, 3.0).position(a);
        assert!(moved.sub(a).length() > 0.5);
        assert!((moved.length() - a.length()).abs() < 1e-12);
    }
}

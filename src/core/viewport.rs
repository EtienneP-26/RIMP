//! View onto the canvas: pan, zoom, rotation and horizontal mirror.

use glam::{Affine2, Mat3, Vec2};

/// Smallest allowed zoom.
pub const MIN_ZOOM: f32 = 0.05;
/// Largest allowed zoom.
pub const MAX_ZOOM: f32 = 64.0;

/// Transform between canvas space and screen space.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Viewport {
    /// Screen position of the canvas origin.
    pub offset: Vec2,
    /// Scale factor (1.0 = 100 %).
    pub zoom: f32,
    /// Rotation in radians.
    pub angle: f32,
    /// Horizontal flip of the canvas.
    pub mirror: bool,
}

impl Default for Viewport {
    /// Identity view.
    fn default() -> Self {
        Self { offset: Vec2::ZERO, zoom: 1.0, angle: 0.0, mirror: false }
    }
}

impl Viewport {
    /// Affine transform canvas -> screen.
    fn affine(&self) -> Affine2 {
        let sx = if self.mirror { -self.zoom } else { self.zoom };
        Affine2::from_scale_angle_translation(Vec2::new(sx, self.zoom), self.angle, self.offset)
    }

    /// Canvas -> screen matrix, usable by a shader.
    pub fn matrix(&self) -> Mat3 {
        Mat3::from(self.affine())
    }

    /// Converts a screen position to a canvas position.
    pub fn screen_to_canvas(&self, p: Vec2) -> Vec2 {
        self.affine().inverse().transform_point2(p)
    }

    /// Converts a canvas position to a screen position.
    pub fn canvas_to_screen(&self, p: Vec2) -> Vec2 {
        self.affine().transform_point2(p)
    }

    /// Multiplies the zoom by `factor` (clamped to `MIN_ZOOM..=MAX_ZOOM`),
    /// keeping the canvas point under `screen_point` fixed.
    pub fn zoom_at(&mut self, screen_point: Vec2, factor: f32) {
        let canvas = self.screen_to_canvas(screen_point);
        self.zoom = (self.zoom * factor).clamp(MIN_ZOOM, MAX_ZOOM);
        self.offset += screen_point - self.canvas_to_screen(canvas);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Asserts a round trip returns the start point.
    fn check(v: Viewport) {
        let p = Vec2::new(123.5, -45.25);
        let back = v.screen_to_canvas(v.canvas_to_screen(p));
        assert!((back - p).length() < 1e-3, "{v:?} -> {back}");
    }

    #[test]
    fn identity_round_trip() {
        let v = Viewport::default();
        assert_eq!(v.canvas_to_screen(Vec2::new(3.0, 4.0)), Vec2::new(3.0, 4.0));
        check(v);
    }

    #[test]
    fn zoom_round_trip() {
        for zoom in [0.05, 0.5, 2.0, 32.0] {
            check(Viewport { zoom, offset: Vec2::new(10.0, 20.0), ..Default::default() });
        }
    }

    #[test]
    fn rotation_round_trip() {
        let v = Viewport { angle: std::f32::consts::FRAC_PI_2, ..Default::default() };
        let s = v.canvas_to_screen(Vec2::new(1.0, 0.0));
        assert!((s - Vec2::new(0.0, 1.0)).length() < 1e-5);
        check(v);
    }

    #[test]
    fn mirror_round_trip() {
        let v = Viewport { mirror: true, ..Default::default() };
        assert_eq!(v.canvas_to_screen(Vec2::new(1.0, 2.0)), Vec2::new(-1.0, 2.0));
        check(v);
    }

    #[test]
    fn combined_round_trip() {
        for angle in [0.3, 1.7, -2.5] {
            let offset = Vec2::new(-80.0, 40.0);
            check(Viewport { offset, zoom: 3.0, angle, mirror: true });
        }
    }

    #[test]
    fn matrix_matches_conversion() {
        let v = Viewport { offset: Vec2::new(5.0, 6.0), zoom: 2.0, angle: 0.8, mirror: true };
        let p = Vec2::new(7.0, 9.0);
        assert!((v.matrix().transform_point2(p) - v.canvas_to_screen(p)).length() < 1e-4);
    }

    /// Asserts the canvas point under the cursor does not move when zooming.
    fn check_fixed(mut v: Viewport, factor: f32) {
        let cursor = Vec2::new(200.0, 150.0);
        let before = v.screen_to_canvas(cursor);
        v.zoom_at(cursor, factor);
        let after = v.screen_to_canvas(cursor);
        assert!((after - before).length() < 1e-2, "{before} vs {after}");
    }

    #[test]
    fn zoom_at_keeps_point_fixed() {
        check_fixed(Viewport::default(), 1.5);
        let offset = Vec2::new(-30.0, 12.0);
        for angle in [0.4, -2.0] {
            check_fixed(Viewport { offset, zoom: 2.0, angle, mirror: true }, 0.7);
            check_fixed(Viewport { offset, zoom: 2.0, angle, mirror: false }, 1.3);
        }
    }

    #[test]
    fn zoom_at_clamps() {
        let mut v = Viewport::default();
        v.zoom_at(Vec2::ZERO, 1000.0);
        assert_eq!(v.zoom, MAX_ZOOM);
        v.zoom_at(Vec2::ZERO, 1e-6);
        assert_eq!(v.zoom, MIN_ZOOM);
    }
}

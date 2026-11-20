//! Input state machine that pans, zooms, rotates and mirrors the view.

use glam::Vec2;

use crate::core::viewport::Viewport;

/// Zoom factor applied per wheel line.
pub const ZOOM_STEP: f32 = 1.1;

/// Tracks the pan triggers and the last cursor position.
#[derive(Default)]
pub struct ViewInput {
    space: bool,
    middle: bool,
    ctrl: bool,
    rotate: bool,
    center: Vec2,
    last: Option<Vec2>,
}

impl ViewInput {
    /// Records whether the Space key is held.
    pub fn set_space(&mut self, held: bool) {
        self.space = held;
    }

    /// Records whether the middle mouse button is held.
    pub fn set_middle(&mut self, held: bool) {
        self.middle = held;
    }

    /// Records whether the Ctrl key is held.
    pub fn set_ctrl(&mut self, held: bool) {
        self.ctrl = held;
    }

    /// Records whether the R key is held.
    pub fn set_rotate(&mut self, held: bool) {
        self.rotate = held;
    }

    /// Sets the screen point the view rotates and mirrors around.
    pub fn set_center(&mut self, center: Vec2) {
        self.center = center;
    }

    /// Flips the view horizontally around the center.
    pub fn toggle_mirror(&self, viewport: &mut Viewport) {
        viewport.toggle_mirror_at(self.center);
    }

    /// Zooms around the last cursor position when Ctrl is held.
    pub fn wheel(&mut self, lines: f32, viewport: &mut Viewport) -> bool {
        match self.last {
            Some(cursor) if self.ctrl && lines != 0.0 => {
                viewport.zoom_at(cursor, ZOOM_STEP.powf(lines));
                true
            }
            _ => false,
        }
    }

    /// Tells whether a pan trigger is active.
    pub fn is_panning(&self) -> bool {
        self.space || self.middle
    }

    /// Tells whether a pan or rotate trigger is active.
    pub fn is_navigating(&self) -> bool {
        self.is_panning() || self.rotate
    }

    /// Angle swept around the center when the cursor goes from `from` to `to`.
    fn swept_angle(&self, from: Vec2, to: Vec2) -> f32 {
        let (a, b) = (from - self.center, to - self.center);
        a.perp_dot(b).atan2(a.dot(b))
    }

    /// Feeds a cursor position and pans or rotates the view if a trigger is active.
    pub fn cursor_moved(&mut self, position: Vec2, viewport: &mut Viewport) -> bool {
        let previous = self.last.replace(position);
        match previous {
            Some(last) if self.is_panning() => {
                viewport.offset += position - last;
                true
            }
            Some(last) if self.rotate => {
                viewport.rotate_about(self.center, self.swept_angle(last, position));
                true
            }
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Moves the cursor from (10, 10) to (15, 7) and returns the result.
    fn drag(input: &mut ViewInput, view: &mut Viewport) -> bool {
        input.cursor_moved(Vec2::new(10.0, 10.0), view);
        input.cursor_moved(Vec2::new(15.0, 7.0), view)
    }

    /// Space plus drag moves the offset.
    #[test]
    fn space_drag_moves_offset() {
        let (mut input, mut view) = (ViewInput::default(), Viewport::default());
        input.set_space(true);
        assert!(drag(&mut input, &mut view));
        assert_eq!(view.offset, Vec2::new(5.0, -3.0));
    }

    /// Releasing Space stops the pan.
    #[test]
    fn release_stops_pan() {
        let (mut input, mut view) = (ViewInput::default(), Viewport::default());
        input.set_space(true);
        drag(&mut input, &mut view);
        input.set_space(false);
        assert!(!input.cursor_moved(Vec2::new(40.0, 40.0), &mut view));
        assert_eq!(view.offset, Vec2::new(5.0, -3.0));
    }

    /// Middle button drag pans.
    #[test]
    fn middle_button_pans() {
        let (mut input, mut view) = (ViewInput::default(), Viewport::default());
        input.set_middle(true);
        assert!(drag(&mut input, &mut view));
        assert_eq!(view.offset, Vec2::new(5.0, -3.0));
    }

    /// No pan happens without a trigger.
    #[test]
    fn no_pan_without_trigger() {
        let (mut input, mut view) = (ViewInput::default(), Viewport::default());
        assert!(!drag(&mut input, &mut view));
        assert_eq!(view.offset, Vec2::ZERO);
    }

    /// Ctrl plus wheel zooms around the cursor.
    #[test]
    fn ctrl_wheel_zooms_at_cursor() {
        let (mut input, mut view) = (ViewInput::default(), Viewport::default());
        input.cursor_moved(Vec2::new(50.0, 40.0), &mut view);
        input.set_ctrl(true);
        assert!(input.wheel(1.0, &mut view));
        assert!((view.zoom - ZOOM_STEP).abs() < 1e-5);
        let fixed = view.screen_to_canvas(Vec2::new(50.0, 40.0));
        assert!((fixed - Vec2::new(50.0, 40.0)).length() < 1e-3);
    }

    /// Wheel without Ctrl leaves the view alone.
    #[test]
    fn wheel_without_ctrl_does_nothing() {
        let (mut input, mut view) = (ViewInput::default(), Viewport::default());
        input.cursor_moved(Vec2::new(50.0, 40.0), &mut view);
        assert!(!input.wheel(1.0, &mut view));
        assert_eq!(view, Viewport::default());
    }

    /// R plus drag rotates by the angle swept around the center.
    #[test]
    fn rotate_drag_turns_view() {
        let (mut input, mut view) = (ViewInput::default(), Viewport::default());
        input.set_rotate(true);
        assert!(input.is_navigating());
        input.cursor_moved(Vec2::new(10.0, 0.0), &mut view);
        assert!(input.cursor_moved(Vec2::new(0.0, 10.0), &mut view));
        assert!((view.angle - std::f32::consts::FRAC_PI_2).abs() < 1e-5);
    }

    /// Releasing R stops the rotation.
    #[test]
    fn release_stops_rotation() {
        let (mut input, mut view) = (ViewInput::default(), Viewport::default());
        input.set_rotate(true);
        input.set_rotate(false);
        input.cursor_moved(Vec2::new(10.0, 0.0), &mut view);
        assert!(!input.cursor_moved(Vec2::new(0.0, 10.0), &mut view));
        assert!(!input.is_navigating());
        assert_eq!(view.angle, 0.0);
    }

    /// Mirroring keeps the center point fixed.
    #[test]
    fn mirror_keeps_center_fixed() {
        let (mut input, mut view) = (ViewInput::default(), Viewport::default());
        let center = Vec2::new(60.0, 40.0);
        input.set_center(center);
        let before = view.screen_to_canvas(center);
        input.toggle_mirror(&mut view);
        assert!(view.mirror);
        assert!((view.screen_to_canvas(center) - before).length() < 1e-4);
    }
}

//! Input state machine that pans the view (Space + drag or middle-button drag).

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

    /// Zooms around the last cursor position when Ctrl is held.
    ///
    /// `lines` is the wheel movement in lines (positive zooms in).
    ///
    /// ## Returns
    /// `true` if the view changed.
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

    /// Feeds a cursor position; moves the view offset if panning.
    ///
    /// ## Returns
    /// `true` if the view moved.
    pub fn cursor_moved(&mut self, position: Vec2, viewport: &mut Viewport) -> bool {
        let previous = self.last.replace(position);
        match previous {
            Some(last) if self.is_panning() => {
                viewport.offset += position - last;
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

    #[test]
    fn space_drag_moves_offset() {
        let (mut input, mut view) = (ViewInput::default(), Viewport::default());
        input.set_space(true);
        assert!(drag(&mut input, &mut view));
        assert_eq!(view.offset, Vec2::new(5.0, -3.0));
    }

    #[test]
    fn release_stops_pan() {
        let (mut input, mut view) = (ViewInput::default(), Viewport::default());
        input.set_space(true);
        drag(&mut input, &mut view);
        input.set_space(false);
        assert!(!input.cursor_moved(Vec2::new(40.0, 40.0), &mut view));
        assert_eq!(view.offset, Vec2::new(5.0, -3.0));
    }

    #[test]
    fn middle_button_pans() {
        let (mut input, mut view) = (ViewInput::default(), Viewport::default());
        input.set_middle(true);
        assert!(drag(&mut input, &mut view));
        assert_eq!(view.offset, Vec2::new(5.0, -3.0));
    }

    #[test]
    fn no_pan_without_trigger() {
        let (mut input, mut view) = (ViewInput::default(), Viewport::default());
        assert!(!drag(&mut input, &mut view));
        assert_eq!(view.offset, Vec2::ZERO);
    }

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

    #[test]
    fn wheel_without_ctrl_does_nothing() {
        let (mut input, mut view) = (ViewInput::default(), Viewport::default());
        input.cursor_moved(Vec2::new(50.0, 40.0), &mut view);
        assert!(!input.wheel(1.0, &mut view));
        assert_eq!(view, Viewport::default());
    }
}

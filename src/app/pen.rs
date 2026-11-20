//! Reads tablet pen events and maps pen positions to canvas space.

use octotablet::Manager;
use octotablet::events::{Event, ToolEvent};

use super::FULL_PRESSURE;
use crate::core::viewport::Viewport;
use glam::Vec2;

/// A pen event reduced to what the drawing needs.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PenEvent {
    Down,
    Up,
    Move { position: (f32, f32), pressure: f32 },
}

/// Collects the pen events received since the last call.
pub fn poll(manager: &mut Manager) -> Vec<PenEvent> {
    let Ok(events) = manager.pump() else {
        return Vec::new();
    };
    events.into_iter().filter_map(event_to_pen).collect()
}

/// Converts a pen position from logical to physical pixels.
pub fn to_physical(position: (f32, f32), scale_factor: f32) -> (f32, f32) {
    (position.0 * scale_factor, position.1 * scale_factor)
}

/// Converts a position on the screen-fitted image to a canvas position.
pub fn to_canvas(position: (f32, f32), viewport: &Viewport) -> (f32, f32) {
    let p = viewport.screen_to_canvas(Vec2::new(position.0, position.1));
    (p.x, p.y)
}

/// Extracts the pen event from a raw tablet event, if any.
fn event_to_pen(event: Event<'_>) -> Option<PenEvent> {
    match event {
        Event::Tool { event, .. } => to_pen_event(event),
        _ => None,
    }
}

/// Keeps only the tool events the drawing cares about.
fn to_pen_event(event: ToolEvent) -> Option<PenEvent> {
    match event {
        ToolEvent::Down => Some(PenEvent::Down),
        ToolEvent::Up => Some(PenEvent::Up),
        ToolEvent::Pose(pose) => Some(PenEvent::Move {
            position: (pose.position[0], pose.position[1]),
            pressure: pose.pressure.get().unwrap_or(FULL_PRESSURE),
        }),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Down and Up events are kept.
    #[test]
    fn down_and_up_are_kept() {
        assert_eq!(to_pen_event(ToolEvent::Down), Some(PenEvent::Down));
        assert_eq!(to_pen_event(ToolEvent::Up), Some(PenEvent::Up));
    }

    /// Positions are multiplied by the scale factor.
    #[test]
    fn position_is_scaled() {
        assert_eq!(to_physical((10.0, 20.0), 2.0), (20.0, 40.0));
        assert_eq!(to_physical((10.0, 20.0), 1.0), (10.0, 20.0));
    }

    /// Positions go through the viewport.
    #[test]
    fn position_goes_through_viewport() {
        assert_eq!(to_canvas((5.0, 6.0), &Viewport::default()), (5.0, 6.0));
        let v = Viewport {
            offset: Vec2::new(10.0, 0.0),
            zoom: 2.0,
            ..Default::default()
        };
        assert_eq!(to_canvas((30.0, 8.0), &v), (10.0, 4.0));
    }

    /// Other tool events are ignored.
    #[test]
    fn other_events_are_ignored() {
        assert_eq!(to_pen_event(ToolEvent::Out), None);
        assert_eq!(to_pen_event(ToolEvent::Frame(None)), None);
    }
}

use octotablet::Manager;
use octotablet::events::{Event, ToolEvent};

use super::FULL_PRESSURE;

/// # A pen event reduced to what the drawing needs.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PenEvent {
    Down,
    Up,
    Move {
        position: (f32, f32),
        pressure: f32,
    },
}

/// # Collects the pen events received since the last call.
///
/// ## Arguments
/// * `manager` - The tablet connection
///
/// ## Returns
/// The events in order. Empty if the tablet reports an error: the mouse still works.
pub fn poll(manager: &mut Manager) -> Vec<PenEvent> {
    let Ok(events) = manager.pump() else {
        return Vec::new();
    };

    events
        .into_iter()
        .filter_map(|event| match event {
            Event::Tool { event, .. } => to_pen_event(event),
            _ => None,
        })
        .collect()
}

/// # Converts a pen position from logical to physical pixels.
///
/// On a high-density screen one logical pixel covers several physical ones.
///
/// ## Arguments
/// * `position` - Position in logical pixels
/// * `scale_factor` - Physical pixels per logical pixel (2.0 on a typical HiDPI screen)
///
/// ## Returns
/// The position in physical pixels.
pub fn to_physical(position: (f32, f32), scale_factor: f32) -> (f32, f32) {
    (position.0 * scale_factor, position.1 * scale_factor)
}

/// # Keeps only the tool events the drawing cares about.
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

    #[test]
    fn down_et_up_sont_gardes() {
        assert_eq!(to_pen_event(ToolEvent::Down), Some(PenEvent::Down));
        assert_eq!(to_pen_event(ToolEvent::Up), Some(PenEvent::Up));
    }

    #[test]
    fn la_position_est_multipliee_par_l_echelle() {
        assert_eq!(to_physical((10.0, 20.0), 2.0), (20.0, 40.0));
        assert_eq!(to_physical((10.0, 20.0), 1.0), (10.0, 20.0));
    }

    #[test]
    fn les_autres_evenements_sont_ignores() {
        assert_eq!(to_pen_event(ToolEvent::Out), None);
        assert_eq!(to_pen_event(ToolEvent::Frame(None)), None);
    }
}

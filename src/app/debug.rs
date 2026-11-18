//! The numbers shown in the debug window.

use std::time::Duration;

/// Pressure, event count and frame time.
#[derive(Default)]
pub struct DebugStats {
    pressure: f32,
    events_pending: usize,
    events_per_frame: usize,
    frame_time: Duration,
}

impl DebugStats {
    /// Notes one drawing event.
    pub fn record_event(&mut self, pressure: f32) {
        self.pressure = pressure;
        self.events_pending += 1;
    }

    /// Starts a frame: counts the events received since the last one.
    pub fn begin_frame(&mut self) {
        self.events_per_frame = self.events_pending;
        self.events_pending = 0;
    }

    /// Ends a frame: remembers how long it took.
    pub fn end_frame(&mut self, frame_time: Duration) {
        self.frame_time = frame_time;
    }

    /// Draws the debug window.
    pub fn show(&self, ctx: &egui::Context) {
        egui::Window::new("Debug").show(ctx, |ui| self.fill(ui));
    }

    /// Writes the labels into the window.
    fn fill(&self, ui: &mut egui::Ui) {
        ui.label(format!("Pression : {:.2}", self.pressure));
        ui.label(format!("Événements par image : {}", self.events_per_frame));
        ui.label(format!(
            "Temps d'une image : {:.2} ms",
            self.frame_time.as_secs_f64() * 1000.0
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Events are counted per frame.
    #[test]
    fn events_are_counted_per_frame() {
        let mut stats = DebugStats::default();
        stats.record_event(0.5);
        stats.record_event(0.6);
        stats.begin_frame();
        assert_eq!(stats.events_per_frame, 2);

        stats.begin_frame();
        assert_eq!(stats.events_per_frame, 0);
    }

    /// The last pressure is kept.
    #[test]
    fn last_pressure_is_kept() {
        let mut stats = DebugStats::default();
        stats.record_event(0.25);
        stats.record_event(0.75);
        assert_eq!(stats.pressure, 0.75);
    }
}

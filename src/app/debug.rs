use std::time::Duration;

/// # The numbers shown in the debug window.
#[derive(Default)]
pub struct DebugStats {
    pressure: f32,
    events_pending: usize,
    events_per_frame: usize,
    frame_time: Duration,
}

impl DebugStats {
    /// # Notes one drawing event (a mouse move or a pen move).
    ///
    /// ## Arguments
    /// * `pressure` - Pen pressure from 0.0 to 1.0
    pub fn record_event(&mut self, pressure: f32) {
        self.pressure = pressure;
        self.events_pending += 1;
    }

    /// # Starts a frame: the events received since the last frame are counted.
    pub fn begin_frame(&mut self) {
        self.events_per_frame = self.events_pending;
        self.events_pending = 0;
    }

    /// # Ends a frame: remembers how long it took to draw.
    pub fn end_frame(&mut self, frame_time: Duration) {
        self.frame_time = frame_time;
    }

    /// # Draws the debug window.
    pub fn show(&self, ctx: &egui::Context) {
        egui::Window::new("Debug").show(ctx, |ui| {
            ui.label(format!("Pression : {:.2}", self.pressure));
            ui.label(format!("Événements par image : {}", self.events_per_frame));
            ui.label(format!(
                "Temps d'une image : {:.2} ms",
                self.frame_time.as_secs_f64() * 1000.0
            ));
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn les_evenements_sont_comptes_par_image() {
        let mut stats = DebugStats::default();
        stats.record_event(0.5);
        stats.record_event(0.6);
        stats.begin_frame();
        assert_eq!(stats.events_per_frame, 2);

        stats.begin_frame();
        assert_eq!(stats.events_per_frame, 0);
    }

    #[test]
    fn la_derniere_pression_est_gardee() {
        let mut stats = DebugStats::default();
        stats.record_event(0.25);
        stats.record_event(0.75);
        assert_eq!(stats.pressure, 0.75);
    }
}

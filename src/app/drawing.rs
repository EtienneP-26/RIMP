use crate::brush::{stamp_disk, stamp_line};
use crate::core::{Pixel, Raster};
use crate::io::pixels_to_rgba8;

/// Radius of the brush, in canvas pixels, at full pressure.
const MAX_RADIUS: f32 = 8.0;

const PAPER: Pixel = Pixel {
    r: 1.0,
    g: 1.0,
    b: 1.0,
    a: 1.0,
};

const INK: Pixel = Pixel {
    r: 0.05,
    g: 0.05,
    b: 0.07,
    a: 1.0,
};

/// # The image being painted and the stroke in progress.
pub struct Drawing {
    raster: Raster,
    last_point: Option<(f32, f32)>,
}

impl Drawing {
    /// # Creates a blank white image.
    ///
    /// ## Arguments
    /// * `width` - Width in pixels
    /// * `height` - Height in pixels
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            raster: Raster::new(width, height, PAPER),
            last_point: None,
        }
    }

    /// # Size of the image, as `(width, height)`.
    pub fn size(&self) -> (usize, usize) {
        (self.raster.width(), self.raster.height())
    }

    /// # The image as RGBA8 bytes, ready to be sent to the GPU.
    pub fn to_rgba8(&self) -> Vec<u8> {
        pixels_to_rgba8(self.raster.pixels())
    }

    /// # Starts a stroke: puts the pen down at a point.
    ///
    /// ## Arguments
    /// * `point` - Position on the image, as `(x, y)` in pixels
    /// * `pressure` - Pen pressure from 0.0 to 1.0, or NaN when there is none (mouse)
    pub fn press(&mut self, point: (f32, f32), pressure: f32) {
        stamp_disk(&mut self.raster, point, radius(pressure), INK);
        self.last_point = Some(point);
    }

    /// # Continues the stroke up to a new point.
    ///
    /// Does nothing if no stroke is in progress.
    ///
    /// ## Arguments
    /// * `point` - Position on the image, as `(x, y)` in pixels
    /// * `pressure` - Pen pressure from 0.0 to 1.0, or NaN when there is none (mouse)
    ///
    /// ## Returns
    /// `true` if the image changed.
    pub fn drag(&mut self, point: (f32, f32), pressure: f32) -> bool {
        let Some(last) = self.last_point else {
            return false;
        };

        stamp_line(&mut self.raster, last, point, radius(pressure), INK);
        self.last_point = Some(point);
        true
    }

    /// # Ends the stroke: lifts the pen.
    pub fn release(&mut self) {
        self.last_point = None;
    }
}

/// # Converts a position on the screen into a position on the image.
///
/// The image is stretched over the whole screen, so this is a simple scale.
///
/// ## Arguments
/// * `position` - Position on the screen, in pixels
/// * `screen` - Screen (window) size, in pixels
/// * `image` - Image size, in pixels
///
/// ## Returns
/// The position on the image.
pub fn screen_to_image(position: (f32, f32), screen: (f32, f32), image: (f32, f32)) -> (f32, f32) {
    (
        position.0 * image.0 / screen.0.max(1.0),
        position.1 * image.1 / screen.1.max(1.0),
    )
}

/// # Brush radius for a pressure; a missing pressure (NaN) counts as full pressure.
fn radius(pressure: f32) -> f32 {
    let pressure = if pressure.is_nan() { 1.0 } else { pressure };
    MAX_RADIUS * pressure.clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pixel_noir(drawing: &Drawing, x: usize, y: usize) -> bool {
        drawing.raster.get(x, y) == Some(INK)
    }

    #[test]
    fn image_neuve_est_blanche() {
        let drawing = Drawing::new(10, 10);
        assert_eq!(drawing.raster.get(5, 5), Some(PAPER));
        assert_eq!(drawing.size(), (10, 10));
    }

    #[test]
    fn press_peint_sous_le_stylet() {
        let mut drawing = Drawing::new(50, 50);
        drawing.press((25.5, 25.5), 1.0);
        assert!(pixel_noir(&drawing, 25, 25));
    }

    #[test]
    fn drag_sans_press_ne_fait_rien() {
        let mut drawing = Drawing::new(50, 50);
        assert!(!drawing.drag((25.5, 25.5), 1.0));
        assert!(!pixel_noir(&drawing, 25, 25));
    }

    #[test]
    fn drag_relie_les_points() {
        let mut drawing = Drawing::new(60, 20);
        drawing.press((10.5, 10.5), 1.0);
        assert!(drawing.drag((50.5, 10.5), 1.0));
        assert!((10..51).all(|x| pixel_noir(&drawing, x, 10)));
    }

    #[test]
    fn release_termine_le_trait() {
        let mut drawing = Drawing::new(50, 50);
        drawing.press((10.5, 10.5), 1.0);
        drawing.release();
        assert!(!drawing.drag((40.5, 40.5), 1.0));
    }

    #[test]
    fn plus_de_pression_donne_un_trait_plus_gros() {
        assert!(radius(1.0) > radius(0.25));
        assert_eq!(radius(0.0), 0.0);
    }

    #[test]
    fn sans_pression_on_prend_la_pression_maximale() {
        assert_eq!(radius(f32::NAN), radius(1.0));
    }

    #[test]
    fn pression_hors_plage_est_bornee() {
        assert_eq!(radius(3.0), radius(1.0));
        assert_eq!(radius(-1.0), 0.0);
    }

    #[test]
    fn conversion_ecran_vers_image_met_a_l_echelle() {
        let point = screen_to_image((400.0, 300.0), (800.0, 600.0), (1024.0, 768.0));
        assert_eq!(point, (512.0, 384.0));
    }

    #[test]
    fn rgba_a_la_bonne_taille() {
        assert_eq!(Drawing::new(4, 3).to_rgba8().len(), 4 * 3 * 4);
    }
}

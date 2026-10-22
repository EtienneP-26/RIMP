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

    /// # Paints at a point, continuing the stroke in progress.
    ///
    /// The first point of a stroke is a disk; the next ones are joined to the previous one
    /// by a line, so a fast stroke leaves no gap.
    ///
    /// ## Arguments
    /// * `point` - Position on the image, as `(x, y)` in pixels
    /// * `pressure` - Pen pressure from 0.0 to 1.0
    pub fn paint(&mut self, point: (f32, f32), pressure: f32) {
        let radius = radius(pressure);

        match self.last_point {
            Some(last) => stamp_line(&mut self.raster, last, point, radius, INK),
            None => stamp_disk(&mut self.raster, point, radius, INK),
        }
        self.last_point = Some(point);
    }

    /// # Ends the stroke: the next `paint` starts a new one.
    pub fn lift(&mut self) {
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

/// # Brush radius for a pressure; an invalid pressure (NaN) counts as full pressure.
fn radius(pressure: f32) -> f32 {
    if pressure.is_nan() {
        MAX_RADIUS
    } else {
        MAX_RADIUS * pressure.clamp(0.0, 1.0)
    }
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
    fn paint_peint_sous_le_stylet() {
        let mut drawing = Drawing::new(50, 50);
        drawing.paint((25.5, 25.5), 1.0);
        assert!(pixel_noir(&drawing, 25, 25));
    }

    #[test]
    fn paint_relie_les_points_du_meme_trait() {
        let mut drawing = Drawing::new(60, 20);
        drawing.paint((10.5, 10.5), 1.0);
        drawing.paint((50.5, 10.5), 1.0);
        assert!((10..51).all(|x| pixel_noir(&drawing, x, 10)));
    }

    #[test]
    fn lift_termine_le_trait() {
        let mut drawing = Drawing::new(60, 20);
        drawing.paint((10.5, 10.5), 1.0);
        drawing.lift();
        drawing.paint((50.5, 10.5), 1.0);
        assert!(!pixel_noir(&drawing, 30, 10));
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

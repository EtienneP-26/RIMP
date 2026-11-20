//! The image being painted and the stroke in progress.

use crate::brush::{stamp_disk, stamp_line};
use crate::core::{Pixel, Raster};
use crate::io::pixels_to_rgba8;

/// Radius of the brush, in canvas pixels, at full pressure.
const MAX_RADIUS: f32 = 8.0;

/// Color of a blank image.
const PAPER: Pixel = Pixel {
    r: 1.0,
    g: 1.0,
    b: 1.0,
    a: 1.0,
};

/// Color of the brush.
const INK: Pixel = Pixel {
    r: 0.05,
    g: 0.05,
    b: 0.07,
    a: 1.0,
};

/// A changed rectangle `(x, y, width, height)` and its RGBA8 bytes.
pub type TileUpdate = ((usize, usize, usize, usize), Vec<u8>);

/// The image and the last point of the stroke in progress.
pub struct Drawing {
    raster: Raster,
    last_point: Option<(f32, f32)>,
}

impl Drawing {
    /// Creates a blank white image.
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            raster: Raster::new(width, height, PAPER),
            last_point: None,
        }
    }

    /// Returns the image size as `(width, height)`.
    pub fn size(&self) -> (usize, usize) {
        (self.raster.width(), self.raster.height())
    }

    /// Returns the image as RGBA8 bytes.
    pub fn to_rgba8(&self) -> Vec<u8> {
        pixels_to_rgba8(&self.raster.pixels())
    }

    /// Drains the tiles painted since the last call, with their RGBA8 bytes.
    pub fn take_dirty_rgba8(&mut self) -> Vec<TileUpdate> {
        let rects = self.raster.take_dirty();
        rects
            .into_iter()
            .map(|rect| self.tile_update(rect))
            .collect()
    }

    /// Paints at a point, joined to the previous point of the stroke.
    pub fn paint(&mut self, point: (f32, f32), pressure: f32) {
        let radius = radius(pressure);

        match self.last_point {
            Some(last) => stamp_line(&mut self.raster, last, point, radius, INK),
            None => stamp_disk(&mut self.raster, point, radius, INK),
        }
        self.last_point = Some(point);
    }

    /// Ends the stroke: the next `paint` starts a new one.
    pub fn lift(&mut self) {
        self.last_point = None;
    }

    /// Builds the RGBA8 update of one rectangle.
    fn tile_update(&self, (x, y, w, h): (usize, usize, usize, usize)) -> TileUpdate {
        let pixels = self.raster.region(x, y, w, h);
        ((x, y, w, h), pixels_to_rgba8(&pixels))
    }
}

/// Converts a screen position into an image position by simple scaling.
pub fn screen_to_image(position: (f32, f32), screen: (f32, f32), image: (f32, f32)) -> (f32, f32) {
    (
        position.0 * image.0 / screen.0.max(1.0),
        position.1 * image.1 / screen.1.max(1.0),
    )
}

/// Returns the brush radius for a pressure; NaN counts as full pressure.
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

    /// Tells whether the pixel holds ink.
    fn is_ink(drawing: &Drawing, x: usize, y: usize) -> bool {
        drawing.raster.get(x, y) == Some(INK)
    }

    /// A new image is white.
    #[test]
    fn new_image_is_white() {
        let drawing = Drawing::new(10, 10);
        assert_eq!(drawing.raster.get(5, 5), Some(PAPER));
        assert_eq!(drawing.size(), (10, 10));
    }

    /// Painting marks the pixel under the pen.
    #[test]
    fn paint_marks_pixel_under_pen() {
        let mut drawing = Drawing::new(50, 50);
        drawing.paint((25.5, 25.5), 1.0);
        assert!(is_ink(&drawing, 25, 25));
    }

    /// Points of one stroke are joined.
    #[test]
    fn paint_joins_points_of_same_stroke() {
        let mut drawing = Drawing::new(60, 20);
        drawing.paint((10.5, 10.5), 1.0);
        drawing.paint((50.5, 10.5), 1.0);
        assert!((10..51).all(|x| is_ink(&drawing, x, 10)));
    }

    /// Lifting the pen ends the stroke.
    #[test]
    fn lift_ends_stroke() {
        let mut drawing = Drawing::new(60, 20);
        drawing.paint((10.5, 10.5), 1.0);
        drawing.lift();
        drawing.paint((50.5, 10.5), 1.0);
        assert!(!is_ink(&drawing, 30, 10));
    }

    /// More pressure gives a bigger radius.
    #[test]
    fn more_pressure_gives_bigger_radius() {
        assert!(radius(1.0) > radius(0.25));
        assert_eq!(radius(0.0), 0.0);
    }

    /// NaN pressure counts as full pressure.
    #[test]
    fn nan_pressure_is_full_pressure() {
        assert_eq!(radius(f32::NAN), radius(1.0));
    }

    /// Out-of-range pressure is clamped.
    #[test]
    fn out_of_range_pressure_is_clamped() {
        assert_eq!(radius(3.0), radius(1.0));
        assert_eq!(radius(-1.0), 0.0);
    }

    /// Screen positions are scaled onto the image.
    #[test]
    fn screen_to_image_scales() {
        let point = screen_to_image((400.0, 300.0), (800.0, 600.0), (1024.0, 768.0));
        assert_eq!(point, (512.0, 384.0));
    }

    /// RGBA8 output has four bytes per pixel.
    #[test]
    fn rgba8_has_expected_size() {
        assert_eq!(Drawing::new(4, 3).to_rgba8().len(), 4 * 3 * 4);
    }
}

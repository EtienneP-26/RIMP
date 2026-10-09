use super::pixel::Pixel;

/// # A `width` × `height` image of pixels kept in memory.
#[derive(Debug, Clone, PartialEq)]
pub struct Raster {
    width: usize,
    height: usize,
    pixels: Vec<Pixel>,
}

impl Raster {
    /// # Creates an image filled with one pixel value.
    ///
    /// ## Arguments
    /// * `width` - Width in pixels
    /// * `height` - Height in pixels
    /// * `fill` - The value of every pixel
    ///
    /// ## Returns
    /// The new image.
    ///
    /// ## Examples
    /// ```
    /// use rimp::core::{Pixel, Raster};
    ///
    /// let white = Pixel { r: 1.0, g: 1.0, b: 1.0, a: 1.0 };
    /// assert_eq!(Raster::new(4, 3, white).get(3, 2), Some(white));
    /// ```
    pub fn new(width: usize, height: usize, fill: Pixel) -> Self {
        Self {
            width,
            height,
            pixels: vec![fill; width * height],
        }
    }

    /// # Width in pixels.
    pub fn width(&self) -> usize {
        self.width
    }

    /// # Height in pixels.
    pub fn height(&self) -> usize {
        self.height
    }

    /// # All the pixels, row by row.
    pub fn pixels(&self) -> &[Pixel] {
        &self.pixels
    }

    /// # Reads the pixel at `(x, y)`.
    ///
    /// ## Returns
    /// The pixel, or `None` if `(x, y)` is outside the image.
    pub fn get(&self, x: usize, y: usize) -> Option<Pixel> {
        (x < self.width && y < self.height).then(|| self.pixels[y * self.width + x])
    }

    /// # Writes the pixel at `(x, y)`.
    ///
    /// A position outside the image is ignored, so a brush can overflow the edges.
    pub fn set(&mut self, x: usize, y: usize, pixel: Pixel) {
        if x < self.width && y < self.height {
            self.pixels[y * self.width + x] = pixel;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ROUGE: Pixel = Pixel {
        r: 1.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };

    #[test]
    fn image_neuve_est_remplie() {
        let image = Raster::new(5, 4, ROUGE);
        assert_eq!((image.width(), image.height()), (5, 4));
        assert_eq!(image.get(4, 3), Some(ROUGE));
        assert_eq!(image.pixels().len(), 20);
    }

    #[test]
    fn set_puis_get_redonne_le_pixel() {
        let mut image = Raster::new(5, 4, Pixel::default());
        image.set(2, 3, ROUGE);
        assert_eq!(image.get(2, 3), Some(ROUGE));
        assert_eq!(image.get(3, 2), Some(Pixel::default()));
    }

    #[test]
    fn get_hors_limites_donne_none() {
        let image = Raster::new(5, 4, ROUGE);
        assert_eq!(image.get(5, 0), None);
        assert_eq!(image.get(0, 4), None);
    }

    #[test]
    fn set_hors_limites_est_ignore() {
        let mut image = Raster::new(5, 4, Pixel::default());
        image.set(5, 0, ROUGE);
        image.set(0, 4, ROUGE);
        assert_eq!(image, Raster::new(5, 4, Pixel::default()));
    }
}

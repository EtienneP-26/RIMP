use super::pixel::Pixel;

pub const TILE_SIZE: usize = 64;

/// # A `TILE_SIZE` × `TILE_SIZE` square/vector of pixels.
#[derive(Debug, Clone, PartialEq)]
pub struct Tile {
    pixels: Vec<Pixel>,
}

impl Tile {
    /// # Creates a fully transparent tile.
    ///
    /// ## Returns
    /// A tile whose pixels are all `Pixel::default()`.
    ///
    /// ## Examples
    /// ```
    /// use rimp::core::{Pixel, Tile};
    ///
    /// assert_eq!(Tile::new().get(0, 0), Pixel::default());
    /// ```
    pub fn new() -> Self {
        Self {
            pixels: vec![Pixel::default(); TILE_SIZE * TILE_SIZE],
        }
    }

    /// # Reads the pixel at position `(x, y)`.
    ///
    /// ## Arguments
    /// * `x` - Column, from 0 to `TILE_SIZE - 1`
    /// * `y` - Row, from 0 to `TILE_SIZE - 1`
    ///
    /// ## Returns
    /// A copy of the pixel.
    ///
    /// ## Panics
    /// If `(x, y)` is outside the tile.
    ///
    /// ## Examples
    /// ```
    /// use rimp::core::{Pixel, Tile};
    ///
    /// let tile = Tile::new();
    /// assert_eq!(tile.get(63, 63), Pixel::default());
    /// ```
    pub fn get(&self, x: usize, y: usize) -> Pixel {
        self.pixels[Self::index(x, y)]
    }

    /// # Writes the pixel at position `(x, y)`.
    ///
    /// ## Arguments
    /// * `x` - Column, from 0 to `TILE_SIZE - 1`
    /// * `y` - Row, from 0 to `TILE_SIZE - 1`
    /// * `pixel` - The new value
    ///
    /// ## Panics
    /// If `(x, y)` is outside the tile.
    ///
    /// ## Examples
    /// ```
    /// use rimp::core::{Pixel, Tile};
    ///
    /// let red = Pixel { r: 1.0, g: 0.0, b: 0.0, a: 1.0 };
    /// let mut tile = Tile::new();
    /// tile.set(10, 20, red);
    /// assert_eq!(tile.get(10, 20), red);
    /// ```
    pub fn set(&mut self, x: usize, y: usize, pixel: Pixel) {
        self.pixels[Self::index(x, y)] = pixel;
    }

    /// # Converts `(x, y)` into an index in the `pixels` array.
    ///
    /// ## Arguments
    /// * `x` - Column
    /// * `y` - Row
    ///
    /// ## Returns
    /// The index of the pixel in `pixels`.
    ///
    /// ## Panics
    /// If `(x, y)` is outside the tile.
    fn index(x: usize, y: usize) -> usize {
        assert!(
            x < TILE_SIZE && y < TILE_SIZE,
            "pixel ({x}, {y}) hors de la tuile"
        );

        y * TILE_SIZE + x
    }
}

impl Default for Tile {
    fn default() -> Self {
        Self::new()
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
    fn tuile_neuve_est_transparente() {
        let tile = Tile::new();
        assert_eq!(tile.get(0, 0), Pixel::default());
        assert_eq!(tile.get(63, 63), Pixel::default());
    }

    #[test]
    fn set_puis_get_redonne_le_pixel() {
        let mut tile = Tile::new();
        tile.set(10, 20, ROUGE);
        assert_eq!(tile.get(10, 20), ROUGE);
    }

    #[test]
    fn set_ne_touche_pas_les_autres_pixels() {
        let mut tile = Tile::new();
        tile.set(10, 20, ROUGE);
        assert_eq!(tile.get(20, 10), Pixel::default());
    }

    #[test]
    fn les_quatre_coins_sont_utilisables() {
        let mut tile = Tile::new();
        for (x, y) in [(0, 0), (63, 0), (0, 63), (63, 63)] {
            tile.set(x, y, ROUGE);
            assert_eq!(tile.get(x, y), ROUGE);
        }
    }

    #[test]
    #[should_panic(expected = "hors de la tuile")]
    fn get_hors_limites_panique() {
        Tile::new().get(TILE_SIZE, 0);
    }

    #[test]
    #[should_panic(expected = "hors de la tuile")]
    fn set_hors_limites_panique() {
        Tile::new().set(0, TILE_SIZE, ROUGE);
    }
}

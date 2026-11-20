//! Fixed size square tile of pixels.

use super::pixel::Pixel;

pub const TILE_SIZE: usize = 64;

/// A `TILE_SIZE` by `TILE_SIZE` square of pixels.
#[derive(Debug, Clone, PartialEq)]
pub struct Tile {
    pixels: Vec<Pixel>,
}

impl Tile {
    /// Creates a fully transparent tile.
    pub fn new() -> Self {
        Self::filled(Pixel::default())
    }

    /// Creates a tile with every pixel set to `pixel`.
    pub fn filled(pixel: Pixel) -> Self {
        Self {
            pixels: vec![pixel; TILE_SIZE * TILE_SIZE],
        }
    }

    /// Reads the pixel at `(x, y)`, panicking if outside the tile.
    pub fn get(&self, x: usize, y: usize) -> Pixel {
        self.pixels[Self::index(x, y)]
    }

    /// Writes the pixel at `(x, y)`, panicking if outside the tile.
    pub fn set(&mut self, x: usize, y: usize, pixel: Pixel) {
        self.pixels[Self::index(x, y)] = pixel;
    }

    /// Converts `(x, y)` into an index in `pixels`, panicking if outside the tile.
    fn index(x: usize, y: usize) -> usize {
        assert!(
            x < TILE_SIZE && y < TILE_SIZE,
            "pixel ({x}, {y}) hors de la tuile"
        );

        y * TILE_SIZE + x
    }
}

impl Default for Tile {
    /// Returns a fully transparent tile.
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const RED: Pixel = Pixel {
        r: 1.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };

    /// A new tile is fully transparent.
    #[test]
    fn new_tile_is_transparent() {
        let tile = Tile::new();
        assert_eq!(tile.get(0, 0), Pixel::default());
        assert_eq!(tile.get(63, 63), Pixel::default());
    }

    /// A filled tile holds the given pixel everywhere.
    #[test]
    fn filled_tile_has_given_value() {
        assert_eq!(Tile::filled(RED).get(5, 7), RED);
    }

    /// Setting then getting returns the same pixel.
    #[test]
    fn set_then_get_returns_pixel() {
        let mut tile = Tile::new();
        tile.set(10, 20, RED);
        assert_eq!(tile.get(10, 20), RED);
    }

    /// Setting a pixel leaves the others untouched.
    #[test]
    fn set_does_not_touch_other_pixels() {
        let mut tile = Tile::new();
        tile.set(10, 20, RED);
        assert_eq!(tile.get(20, 10), Pixel::default());
    }

    /// All four corners can be written and read.
    #[test]
    fn four_corners_are_usable() {
        let mut tile = Tile::new();
        for (x, y) in [(0, 0), (63, 0), (0, 63), (63, 63)] {
            tile.set(x, y, RED);
            assert_eq!(tile.get(x, y), RED);
        }
    }

    /// Reading outside the tile panics.
    #[test]
    #[should_panic(expected = "hors de la tuile")]
    fn get_out_of_bounds_panics() {
        Tile::new().get(TILE_SIZE, 0);
    }

    /// Writing outside the tile panics.
    #[test]
    #[should_panic(expected = "hors de la tuile")]
    fn set_out_of_bounds_panics() {
        Tile::new().set(0, TILE_SIZE, RED);
    }
}

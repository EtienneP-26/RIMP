//! A sparse tiled image that reads unwritten tiles as a background colour.

use std::collections::BTreeSet;

use super::pixel::Pixel;
use super::tile::{TILE_SIZE, Tile};

/// A sparse grid of tiles; a tile never written is `None` and reads as the background.
#[derive(Debug, Clone, PartialEq)]
pub struct Raster {
    width: usize,
    height: usize,
    background: Pixel,
    tiles_per_row: usize,
    tiles: Vec<Option<Tile>>,
    dirty: BTreeSet<usize>,
}

impl Raster {
    /// Creates an image whose pixels all read as `fill`, with no tile allocated.
    pub fn new(width: usize, height: usize, fill: Pixel) -> Self {
        let tiles_per_row = width.div_ceil(TILE_SIZE);
        let tiles_per_column = height.div_ceil(TILE_SIZE);
        Self {
            width,
            height,
            background: fill,
            tiles_per_row,
            tiles: vec![None; tiles_per_row * tiles_per_column],
            dirty: BTreeSet::new(),
        }
    }

    /// Width in pixels.
    pub fn width(&self) -> usize {
        self.width
    }

    /// Height in pixels.
    pub fn height(&self) -> usize {
        self.height
    }

    /// Number of tiles allocated so far.
    pub fn allocated_tiles(&self) -> usize {
        self.tiles.iter().flatten().count()
    }

    /// All the pixels, row by row (empty tiles read as the background).
    pub fn pixels(&self) -> Vec<Pixel> {
        self.region(0, 0, self.width, self.height)
    }

    /// The pixels of a rectangle, row by row; it must lie inside the image.
    pub fn region(&self, x: usize, y: usize, w: usize, h: usize) -> Vec<Pixel> {
        let coords = (y..y + h).flat_map(|py| (x..x + w).map(move |px| (px, py)));
        coords.map(|(px, py)| self.read(px, py)).collect()
    }

    /// Drains the tiles written since the last call as clipped pixel rectangles.
    pub fn take_dirty(&mut self) -> Vec<(usize, usize, usize, usize)> {
        let dirty = std::mem::take(&mut self.dirty);
        dirty
            .into_iter()
            .map(|index| self.tile_rect(index))
            .collect()
    }

    /// Reads the pixel at `(x, y)`, or `None` when outside the image.
    pub fn get(&self, x: usize, y: usize) -> Option<Pixel> {
        (x < self.width && y < self.height).then(|| self.read(x, y))
    }

    /// Writes a pixel, allocating its tile on first write; out-of-bounds writes are ignored.
    pub fn set(&mut self, x: usize, y: usize, pixel: Pixel) {
        if x >= self.width || y >= self.height {
            return;
        }
        let index = self.tile_index(x, y);
        self.dirty.insert(index);
        self.tiles[index]
            .get_or_insert_with(|| Tile::filled(self.background))
            .set(x % TILE_SIZE, y % TILE_SIZE, pixel);
    }

    /// Pixel rectangle `(x, y, width, height)` covered by a tile, clipped at the edges.
    fn tile_rect(&self, index: usize) -> (usize, usize, usize, usize) {
        let x = (index % self.tiles_per_row) * TILE_SIZE;
        let y = (index / self.tiles_per_row) * TILE_SIZE;
        let w = TILE_SIZE.min(self.width - x);
        let h = TILE_SIZE.min(self.height - y);
        (x, y, w, h)
    }

    /// Index of the tile holding `(x, y)`.
    fn tile_index(&self, x: usize, y: usize) -> usize {
        (y / TILE_SIZE) * self.tiles_per_row + x / TILE_SIZE
    }

    /// Reads an in-bounds pixel.
    fn read(&self, x: usize, y: usize) -> Pixel {
        match &self.tiles[self.tile_index(x, y)] {
            Some(tile) => tile.get(x % TILE_SIZE, y % TILE_SIZE),
            None => self.background,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Opaque red used by the tests.
    const RED: Pixel = Pixel {
        r: 1.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };

    /// A new image reads as its fill colour.
    #[test]
    fn new_image_is_filled() {
        let image = Raster::new(5, 4, RED);
        assert_eq!((image.width(), image.height()), (5, 4));
        assert_eq!(image.get(4, 3), Some(RED));
        assert_eq!(image.pixels().len(), 20);
    }

    /// Writing a pixel then reading it gives it back.
    #[test]
    fn set_then_get_returns_pixel() {
        let mut image = Raster::new(5, 4, Pixel::default());
        image.set(2, 3, RED);
        assert_eq!(image.get(2, 3), Some(RED));
        assert_eq!(image.get(3, 2), Some(Pixel::default()));
    }

    /// Reading outside the image gives `None`.
    #[test]
    fn get_out_of_bounds_is_none() {
        let image = Raster::new(5, 4, RED);
        assert_eq!(image.get(5, 0), None);
        assert_eq!(image.get(0, 4), None);
    }

    /// Writing outside the image changes nothing.
    #[test]
    fn set_out_of_bounds_is_ignored() {
        let mut image = Raster::new(5, 4, Pixel::default());
        image.set(5, 0, RED);
        image.set(0, 4, RED);
        assert_eq!(image, Raster::new(5, 4, Pixel::default()));
    }

    /// A tile is allocated only on its first write.
    #[test]
    fn tile_allocated_on_first_write() {
        let mut image = Raster::new(200, 200, Pixel::default());
        assert_eq!(image.allocated_tiles(), 0);
        image.set(70, 5, RED);
        assert_eq!(image.allocated_tiles(), 1);
        image.set(71, 6, RED);
        assert_eq!(image.allocated_tiles(), 1);
    }

    /// Writes on both sides of a tile border use two tiles.
    #[test]
    fn writes_across_tile_border() {
        let mut image = Raster::new(200, 200, Pixel::default());
        image.set(63, 63, RED);
        image.set(64, 64, RED);
        assert_eq!(image.allocated_tiles(), 2);
        assert_eq!(image.get(63, 63), Some(RED));
        assert_eq!(image.get(64, 64), Some(RED));
        assert_eq!(image.get(64, 63), Some(Pixel::default()));
    }

    /// An out-of-bounds write allocates no tile.
    #[test]
    fn out_of_bounds_write_allocates_nothing() {
        let mut image = Raster::new(100, 100, Pixel::default());
        image.set(100, 0, RED);
        image.set(0, 100, RED);
        assert_eq!(image.allocated_tiles(), 0);
    }

    /// A write marks the tile that holds it as dirty.
    #[test]
    fn write_marks_right_tile() {
        let mut image = Raster::new(200, 100, Pixel::default());
        image.set(70, 5, RED);
        assert_eq!(image.take_dirty(), vec![(64, 0, 64, 64)]);
    }

    /// Taking the dirty tiles empties the set.
    #[test]
    fn take_dirty_empties_set() {
        let mut image = Raster::new(200, 100, Pixel::default());
        image.set(1, 1, RED);
        assert_eq!(image.take_dirty().len(), 1);
        assert!(image.take_dirty().is_empty());
    }

    /// Writes on both sides of a border mark two tiles.
    #[test]
    fn border_writes_mark_two_tiles() {
        let mut image = Raster::new(200, 100, Pixel::default());
        image.set(63, 5, RED);
        image.set(64, 5, RED);
        assert_eq!(image.take_dirty().len(), 2);
    }

    /// A tile at the image edge is clipped.
    #[test]
    fn edge_tile_is_clipped() {
        let mut image = Raster::new(200, 100, Pixel::default());
        image.set(199, 99, RED);
        assert_eq!(image.take_dirty(), vec![(192, 64, 8, 36)]);
    }

    /// An out-of-bounds write marks no tile.
    #[test]
    fn out_of_bounds_write_marks_nothing() {
        let mut image = Raster::new(100, 100, Pixel::default());
        image.set(100, 0, RED);
        assert!(image.take_dirty().is_empty());
    }
}

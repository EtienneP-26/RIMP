use std::collections::BTreeSet;

use super::pixel::Pixel;
use super::tile::{TILE_SIZE, Tile};

/// # A `width` × `height` image made of a sparse grid of tiles.
///
/// A tile that was never written is `None` and takes no memory: it reads as `background`.
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
    /// # Creates an image whose pixels all read as one value, with no tile allocated.
    ///
    /// ## Arguments
    /// * `width` - Width in pixels
    /// * `height` - Height in pixels
    /// * `fill` - The value of every pixel not written yet
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

    /// # Width in pixels.
    pub fn width(&self) -> usize {
        self.width
    }

    /// # Height in pixels.
    pub fn height(&self) -> usize {
        self.height
    }

    /// # Number of tiles allocated so far.
    pub fn allocated_tiles(&self) -> usize {
        self.tiles.iter().flatten().count()
    }

    /// # All the pixels, row by row (empty tiles read as the background).
    pub fn pixels(&self) -> Vec<Pixel> {
        self.region(0, 0, self.width, self.height)
    }

    /// # The pixels of a rectangle, row by row; it must lie inside the image.
    pub fn region(&self, x: usize, y: usize, w: usize, h: usize) -> Vec<Pixel> {
        let coords = (y..y + h).flat_map(|py| (x..x + w).map(move |px| (px, py)));
        coords.map(|(px, py)| self.read(px, py)).collect()
    }

    /// # Drains the tiles written since the last call.
    ///
    /// ## Returns
    /// One `(x, y, width, height)` rectangle per tile, in pixels, clipped at the image edges.
    pub fn take_dirty(&mut self) -> Vec<(usize, usize, usize, usize)> {
        let dirty = std::mem::take(&mut self.dirty);
        dirty.into_iter().map(|index| self.tile_rect(index)).collect()
    }

    /// # Pixel rectangle `(x, y, width, height)` covered by a tile, clipped at the edges.
    fn tile_rect(&self, index: usize) -> (usize, usize, usize, usize) {
        let x = (index % self.tiles_per_row) * TILE_SIZE;
        let y = (index / self.tiles_per_row) * TILE_SIZE;
        let w = TILE_SIZE.min(self.width - x);
        let h = TILE_SIZE.min(self.height - y);
        (x, y, w, h)
    }

    /// # Reads the pixel at `(x, y)`.
    ///
    /// ## Returns
    /// The pixel, or `None` if `(x, y)` is outside the image.
    pub fn get(&self, x: usize, y: usize) -> Option<Pixel> {
        (x < self.width && y < self.height).then(|| self.read(x, y))
    }

    /// # Writes the pixel at `(x, y)`, allocating its tile on first write.
    ///
    /// A position outside the image is ignored, so a brush can overflow the edges.
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

    /// # Index of the tile holding `(x, y)`.
    fn tile_index(&self, x: usize, y: usize) -> usize {
        (y / TILE_SIZE) * self.tiles_per_row + x / TILE_SIZE
    }

    /// # Reads an in-bounds pixel.
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

    #[test]
    fn tuile_allouee_a_la_premiere_ecriture() {
        let mut image = Raster::new(200, 200, Pixel::default());
        assert_eq!(image.allocated_tiles(), 0);
        image.set(70, 5, ROUGE);
        assert_eq!(image.allocated_tiles(), 1);
        image.set(71, 6, ROUGE);
        assert_eq!(image.allocated_tiles(), 1);
    }

    #[test]
    fn ecriture_de_part_et_d_autre_d_une_frontiere() {
        let mut image = Raster::new(200, 200, Pixel::default());
        image.set(63, 63, ROUGE);
        image.set(64, 64, ROUGE);
        assert_eq!(image.allocated_tiles(), 2);
        assert_eq!(image.get(63, 63), Some(ROUGE));
        assert_eq!(image.get(64, 64), Some(ROUGE));
        assert_eq!(image.get(64, 63), Some(Pixel::default()));
    }

    #[test]
    fn ecriture_hors_limites_n_alloue_rien() {
        let mut image = Raster::new(100, 100, Pixel::default());
        image.set(100, 0, ROUGE);
        image.set(0, 100, ROUGE);
        assert_eq!(image.allocated_tiles(), 0);
    }

    #[test]
    fn ecriture_marque_la_bonne_tuile() {
        let mut image = Raster::new(200, 100, Pixel::default());
        image.set(70, 5, ROUGE);
        assert_eq!(image.take_dirty(), vec![(64, 0, 64, 64)]);
    }

    #[test]
    fn take_dirty_vide_l_ensemble() {
        let mut image = Raster::new(200, 100, Pixel::default());
        image.set(1, 1, ROUGE);
        assert_eq!(image.take_dirty().len(), 1);
        assert!(image.take_dirty().is_empty());
    }

    #[test]
    fn ecriture_a_la_frontiere_marque_deux_tuiles() {
        let mut image = Raster::new(200, 100, Pixel::default());
        image.set(63, 5, ROUGE);
        image.set(64, 5, ROUGE);
        assert_eq!(image.take_dirty().len(), 2);
    }

    #[test]
    fn tuile_du_bord_est_rognee() {
        let mut image = Raster::new(200, 100, Pixel::default());
        image.set(199, 99, ROUGE);
        assert_eq!(image.take_dirty(), vec![(192, 64, 8, 36)]);
    }

    #[test]
    fn ecriture_hors_limites_ne_marque_rien() {
        let mut image = Raster::new(100, 100, Pixel::default());
        image.set(100, 0, ROUGE);
        assert!(image.take_dirty().is_empty());
    }
}

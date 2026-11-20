//! Brush stamps: filled disks and gap-free lines of disks.

use crate::core::{Pixel, Raster};

/// Paints a filled disk of `radius` centred on `center`.
pub fn stamp_disk(image: &mut Raster, center: (f32, f32), radius: f32, color: Pixel) {
    let (cx, cy) = center;
    let x_range = clamp_range(cx - radius, cx + radius, image.width());
    let y_range = clamp_range(cy - radius, cy + radius, image.height());

    for y in y_range {
        for x in x_range.clone() {
            let (dx, dy) = (x as f32 + 0.5 - cx, y as f32 + 0.5 - cy);
            if dx * dx + dy * dy < radius * radius {
                image.set(x, y, color);
            }
        }
    }
}

/// Paints disks from one point to another with no gap.
pub fn stamp_line(image: &mut Raster, from: (f32, f32), to: (f32, f32), radius: f32, color: Pixel) {
    let (dx, dy) = (to.0 - from.0, to.1 - from.1);
    let step = (radius / 2.0).max(0.5);
    let count = ((dx * dx + dy * dy).sqrt() / step).ceil().max(1.0) as usize;

    for i in 0..=count {
        let t = i as f32 / count as f32;
        stamp_disk(image, (from.0 + dx * t, from.1 + dy * t), radius, color);
    }
}

/// Returns the pixel indices covered by `[min, max]`, kept inside `0..limit`.
fn clamp_range(min: f32, max: f32, limit: usize) -> std::ops::Range<usize> {
    let start = min.floor().max(0.0) as usize;
    let end = (max.ceil().max(0.0) as usize).min(limit);
    start..end
}

#[cfg(test)]
mod tests {
    use super::*;

    const BLACK: Pixel = Pixel {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };

    /// Builds a transparent image of the given size.
    fn empty(width: usize, height: usize) -> Raster {
        Raster::new(width, height, Pixel::default())
    }

    /// Checks a disk paints its centre but not the corners.
    #[test]
    fn disk_paints_center_but_not_corners() {
        let mut image = empty(11, 11);
        stamp_disk(&mut image, (5.5, 5.5), 3.0, BLACK);
        assert_eq!(image.get(5, 5), Some(BLACK));
        assert_eq!(image.get(0, 0), Some(Pixel::default()));
        assert_eq!(image.get(8, 8), Some(Pixel::default()));
    }

    /// Checks a bigger disk covers more pixels.
    #[test]
    fn bigger_disk_covers_more_pixels() {
        let count = |radius| {
            let mut image = empty(21, 21);
            stamp_disk(&mut image, (10.5, 10.5), radius, BLACK);
            image.pixels().iter().filter(|p| **p == BLACK).count()
        };
        assert!(count(5.0) > count(2.0));
    }

    /// Checks a zero radius paints nothing.
    #[test]
    fn zero_radius_disk_paints_nothing() {
        let mut image = empty(5, 5);
        stamp_disk(&mut image, (2.5, 2.5), 0.0, BLACK);
        assert_eq!(image, empty(5, 5));
    }

    /// Checks a disk past the edges does not panic.
    #[test]
    fn disk_overflowing_edges_does_not_panic() {
        let mut image = empty(5, 5);
        stamp_disk(&mut image, (-3.0, 100.0), 10.0, BLACK);
        stamp_disk(&mut image, (0.0, 0.0), 3.0, BLACK);
        assert_eq!(image.get(0, 0), Some(BLACK));
    }

    /// Checks a line joins both points with no gap.
    #[test]
    fn line_joins_both_points_without_gap() {
        let mut image = empty(40, 5);
        stamp_line(&mut image, (2.5, 2.5), (37.5, 2.5), 1.0, BLACK);
        for x in 2..38 {
            assert_eq!(image.get(x, 2), Some(BLACK), "gap at x = {x}");
        }
    }

    /// Checks a zero-length line stamps one disk.
    #[test]
    fn zero_length_line_stamps_one_disk() {
        let mut image = empty(5, 5);
        stamp_line(&mut image, (2.5, 2.5), (2.5, 2.5), 1.0, BLACK);
        assert_eq!(image.get(2, 2), Some(BLACK));
    }
}

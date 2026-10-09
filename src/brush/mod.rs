use crate::core::{Pixel, Raster};

/// # Paints a filled disk.
///
/// ## Arguments
/// * `image` - The image to paint on
/// * `center` - Centre of the disk, as `(x, y)` in pixels
/// * `radius` - Radius in pixels
/// * `color` - The paint
///
/// ## Examples
/// ```
/// use rimp::brush::stamp_disk;
/// use rimp::core::{Pixel, Raster};
///
/// let black = Pixel { r: 0.0, g: 0.0, b: 0.0, a: 1.0 };
/// let mut image = Raster::new(9, 9, Pixel::default());
/// stamp_disk(&mut image, (4.0, 4.0), 2.0, black);
/// assert_eq!(image.get(4, 4), Some(black));
/// ```
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

/// # Paints a line of disks from one point to another, with no gap.
///
/// ## Arguments
/// * `image` - The image to paint on
/// * `from` - Start point, as `(x, y)` in pixels
/// * `to` - End point, as `(x, y)` in pixels
/// * `radius` - Radius of the disks in pixels
/// * `color` - The paint
pub fn stamp_line(image: &mut Raster, from: (f32, f32), to: (f32, f32), radius: f32, color: Pixel) {
    let (dx, dy) = (to.0 - from.0, to.1 - from.1);
    let step = (radius / 2.0).max(0.5);
    let count = ((dx * dx + dy * dy).sqrt() / step).ceil().max(1.0) as usize;

    for i in 0..=count {
        let t = i as f32 / count as f32;
        stamp_disk(image, (from.0 + dx * t, from.1 + dy * t), radius, color);
    }
}

/// # The pixel indices covered by `[min, max]`, kept inside `0..limit`.
fn clamp_range(min: f32, max: f32, limit: usize) -> std::ops::Range<usize> {
    let start = min.floor().max(0.0) as usize;
    let end = (max.ceil().max(0.0) as usize).min(limit);
    start..end
}

#[cfg(test)]
mod tests {
    use super::*;

    const NOIR: Pixel = Pixel {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };

    fn vide(width: usize, height: usize) -> Raster {
        Raster::new(width, height, Pixel::default())
    }

    #[test]
    fn disque_peint_son_centre_mais_pas_les_coins() {
        let mut image = vide(11, 11);
        stamp_disk(&mut image, (5.5, 5.5), 3.0, NOIR);
        assert_eq!(image.get(5, 5), Some(NOIR));
        assert_eq!(image.get(0, 0), Some(Pixel::default()));
        assert_eq!(image.get(8, 8), Some(Pixel::default()));
    }

    #[test]
    fn disque_plus_grand_couvre_plus_de_pixels() {
        let compter = |rayon| {
            let mut image = vide(21, 21);
            stamp_disk(&mut image, (10.5, 10.5), rayon, NOIR);
            image.pixels().iter().filter(|p| **p == NOIR).count()
        };
        assert!(compter(5.0) > compter(2.0));
    }

    #[test]
    fn disque_rayon_zero_ne_peint_rien() {
        let mut image = vide(5, 5);
        stamp_disk(&mut image, (2.5, 2.5), 0.0, NOIR);
        assert_eq!(image, vide(5, 5));
    }

    #[test]
    fn disque_deborde_sans_paniquer() {
        let mut image = vide(5, 5);
        stamp_disk(&mut image, (-3.0, 100.0), 10.0, NOIR);
        stamp_disk(&mut image, (0.0, 0.0), 3.0, NOIR);
        assert_eq!(image.get(0, 0), Some(NOIR));
    }

    #[test]
    fn ligne_relie_les_deux_points_sans_trou() {
        let mut image = vide(40, 5);
        stamp_line(&mut image, (2.5, 2.5), (37.5, 2.5), 1.0, NOIR);
        for x in 2..38 {
            assert_eq!(image.get(x, 2), Some(NOIR), "trou en x = {x}");
        }
    }

    #[test]
    fn ligne_de_longueur_nulle_pose_un_disque() {
        let mut image = vide(5, 5);
        stamp_line(&mut image, (2.5, 2.5), (2.5, 2.5), 1.0, NOIR);
        assert_eq!(image.get(2, 2), Some(NOIR));
    }
}

use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

use crate::core::{Pixel, TILE_SIZE, Tile};
use png::{BitDepth, ColorType, Encoder, EncodingError};

/// # Converts a float channel (0.0 to 1.0) into a byte.
///
/// ## Arguments
/// * `value` - The channel value, clamped to 0.0..=1.0
///
/// ## Returns
/// The channel as a `u8`, rounded to the nearest value.
fn to_byte(value: f32) -> u8 {
    (value.clamp(0.0, 1.0) * 255.0).round() as u8
}

/// # Converts a premultiplied `Pixel` into straight RGBA bytes.
///
/// PNG stores straight (non-premultiplied) alpha, so the colour
/// channels are divided by alpha. A fully transparent pixel gives
/// `[0, 0, 0, 0]`.
///
/// ## Arguments
/// * `pixel` - A premultiplied pixel
///
/// ## Returns
/// The `[r, g, b, a]` bytes.
///
/// ## Examples
/// ```
/// use rimp::core::Pixel;
/// use rimp::io::pixel_to_rgba8;
///
/// let half_red = Pixel { r: 0.5, g: 0.0, b: 0.0, a: 0.5 };
/// assert_eq!(pixel_to_rgba8(half_red), [255, 0, 0, 128]);
/// ```
pub fn pixel_to_rgba8(pixel: Pixel) -> [u8; 4] {
    if pixel.a <= 0.0 {
        return [0, 0, 0, 0];
    }

    [
        to_byte(pixel.r / pixel.a),
        to_byte(pixel.g / pixel.a),
        to_byte(pixel.b / pixel.a),
        to_byte(pixel.a),
    ]
}

/// # Converts a tile into a flat RGBA8 buffer, row by row.
///
/// ## Arguments
/// * `tile` - The tile to convert
///
/// ## Returns
/// A buffer of `TILE_SIZE * TILE_SIZE * 4` bytes.
pub fn tile_to_rgba8(tile: &Tile) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(TILE_SIZE * TILE_SIZE * 4);

    for y in 0..TILE_SIZE {
        for x in 0..TILE_SIZE {
            bytes.extend_from_slice(&pixel_to_rgba8(tile.get(x, y)));
        }
    }

    bytes
}

/// # Writes a tile to a PNG file.
///
/// ## Arguments
/// * `tile` - The tile to export
/// * `path` - Where to write the file
///
/// ## Returns
/// `Ok(())` once the file is written.
///
/// ## Errors
/// If the file cannot be created or the PNG cannot be encoded.
///
/// ## Examples
/// ```no_run
/// use rimp::core::Tile;
/// use rimp::io::export_tile_png;
///
/// export_tile_png(&Tile::new(), "tile.png").unwrap();
/// ```
pub fn export_tile_png(tile: &Tile, path: impl AsRef<Path>) -> Result<(), EncodingError> {
    let file = BufWriter::new(File::create(path)?);
    let mut encoder = Encoder::new(file, TILE_SIZE as u32, TILE_SIZE as u32);

    encoder.set_color(ColorType::Rgba);
    encoder.set_depth(BitDepth::Eight);

    encoder
        .write_header()?
        .write_image_data(&tile_to_rgba8(tile))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn px(r: f32, g: f32, b: f32, a: f32) -> Pixel {
        Pixel { r, g, b, a }
    }

    #[test]
    fn pixel_opaque_garde_ses_couleurs() {
        assert_eq!(pixel_to_rgba8(px(1.0, 0.0, 0.0, 1.0)), [255, 0, 0, 255]);
    }

    #[test]
    fn pixel_transparent_donne_zero() {
        assert_eq!(pixel_to_rgba8(Pixel::default()), [0, 0, 0, 0]);
    }

    #[test]
    fn pixel_semi_transparent_est_depremultiplie() {
        assert_eq!(pixel_to_rgba8(px(0.5, 0.0, 0.0, 0.5)), [255, 0, 0, 128]);
    }

    #[test]
    fn valeurs_hors_plage_sont_bornees() {
        assert_eq!(pixel_to_rgba8(px(2.0, 0.0, 0.0, 1.0)), [255, 0, 0, 255]);
    }

    #[test]
    fn buffer_a_la_bonne_taille() {
        assert_eq!(tile_to_rgba8(&Tile::new()).len(), TILE_SIZE * TILE_SIZE * 4);
    }

    #[test]
    fn pixel_place_au_bon_endroit_dans_le_buffer() {
        let mut tile = Tile::new();
        tile.set(2, 1, px(0.0, 1.0, 0.0, 1.0));

        let bytes = tile_to_rgba8(&tile);
        let start = (TILE_SIZE + 2) * 4;
        assert_eq!(&bytes[start..start + 4], &[0, 255, 0, 255]);
    }

    #[test]
    fn export_ecrit_un_png_relisible() {
        let mut tile = Tile::new();
        tile.set(0, 0, px(1.0, 0.0, 0.0, 1.0));

        let path = std::env::temp_dir().join("rimp_export_test.png");
        export_tile_png(&tile, &path).unwrap();

        let decoder = png::Decoder::new(std::io::BufReader::new(File::open(&path).unwrap()));
        let mut reader = decoder.read_info().unwrap();
        let mut buf = vec![0; reader.output_buffer_size().unwrap()];
        let info = reader.next_frame(&mut buf).unwrap();

        assert_eq!(
            (info.width, info.height),
            (TILE_SIZE as u32, TILE_SIZE as u32)
        );
        assert_eq!(&buf[..4], &[255, 0, 0, 255]);

        std::fs::remove_file(path).unwrap();
    }
}

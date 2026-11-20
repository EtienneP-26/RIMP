//! PNG export and RGBA8 conversion of pixels and tiles.

use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

use crate::core::{Pixel, TILE_SIZE, Tile};
use png::{BitDepth, ColorType, Encoder, EncodingError};

/// Converts a float channel in 0.0..=1.0 into a byte.
fn to_byte(value: f32) -> u8 {
    (value.clamp(0.0, 1.0) * 255.0).round() as u8
}

/// Converts a premultiplied pixel into straight RGBA bytes.
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

/// Converts a list of pixels into a flat RGBA8 buffer.
pub fn pixels_to_rgba8(pixels: &[Pixel]) -> Vec<u8> {
    pixels.iter().flat_map(|p| pixel_to_rgba8(*p)).collect()
}

/// Converts a tile into a flat RGBA8 buffer, row by row.
pub fn tile_to_rgba8(tile: &Tile) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(TILE_SIZE * TILE_SIZE * 4);

    for y in 0..TILE_SIZE {
        for x in 0..TILE_SIZE {
            bytes.extend_from_slice(&pixel_to_rgba8(tile.get(x, y)));
        }
    }

    bytes
}

/// Writes a tile to a PNG file.
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

    /// Builds a pixel from its four channels.
    fn px(r: f32, g: f32, b: f32, a: f32) -> Pixel {
        Pixel { r, g, b, a }
    }

    /// Checks an opaque pixel keeps its colors.
    #[test]
    fn opaque_pixel_keeps_its_colors() {
        assert_eq!(pixel_to_rgba8(px(1.0, 0.0, 0.0, 1.0)), [255, 0, 0, 255]);
    }

    /// Checks a transparent pixel gives zeros.
    #[test]
    fn transparent_pixel_gives_zeros() {
        assert_eq!(pixel_to_rgba8(Pixel::default()), [0, 0, 0, 0]);
    }

    /// Checks a half-transparent pixel is unpremultiplied.
    #[test]
    fn semi_transparent_pixel_is_unpremultiplied() {
        assert_eq!(pixel_to_rgba8(px(0.5, 0.0, 0.0, 0.5)), [255, 0, 0, 128]);
    }

    /// Checks out-of-range channels are clamped.
    #[test]
    fn out_of_range_values_are_clamped() {
        assert_eq!(pixel_to_rgba8(px(2.0, 0.0, 0.0, 1.0)), [255, 0, 0, 255]);
    }

    /// Checks each pixel of a list yields four bytes.
    #[test]
    fn pixel_list_gives_four_bytes_each() {
        let pixels = [px(1.0, 0.0, 0.0, 1.0), Pixel::default()];
        assert_eq!(pixels_to_rgba8(&pixels), [255, 0, 0, 255, 0, 0, 0, 0]);
    }

    /// Checks the tile buffer has the expected length.
    #[test]
    fn buffer_has_the_right_size() {
        assert_eq!(tile_to_rgba8(&Tile::new()).len(), TILE_SIZE * TILE_SIZE * 4);
    }

    /// Checks a pixel is written at the right offset.
    #[test]
    fn pixel_lands_at_the_right_buffer_offset() {
        let mut tile = Tile::new();
        tile.set(2, 1, px(0.0, 1.0, 0.0, 1.0));

        let bytes = tile_to_rgba8(&tile);
        let start = (TILE_SIZE + 2) * 4;
        assert_eq!(&bytes[start..start + 4], &[0, 255, 0, 255]);
    }

    /// Checks the exported PNG can be read back.
    #[test]
    fn export_writes_a_readable_png() {
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

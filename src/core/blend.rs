//! Blend modes combining two premultiplied pixels.

use super::pixel::{Pixel, blend_normal};

/// How an upper pixel is combined with a lower one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BlendMode {
    #[default]
    Normal,
    Multiply,
    Add,
}

/// Blends two premultiplied pixels with the given mode.
pub fn blend(mode: BlendMode, src: Pixel, dst: Pixel) -> Pixel {
    match mode {
        BlendMode::Normal => blend_normal(src, dst),
        BlendMode::Multiply => blend_multiply(src, dst),
        BlendMode::Add => blend_add(src, dst),
    }
}

/// Multiply blend on premultiplied pixels, alpha composed as in normal mode.
fn blend_multiply(src: Pixel, dst: Pixel) -> Pixel {
    let channel = |s: f32, d: f32| s * (1.0 - dst.a) + d * (1.0 - src.a) + s * d;

    Pixel {
        r: channel(src.r, dst.r),
        g: channel(src.g, dst.g),
        b: channel(src.b, dst.b),
        a: src.a + dst.a * (1.0 - src.a),
    }
}

/// Additive blend on premultiplied pixels, each channel capped at 1.0.
fn blend_add(src: Pixel, dst: Pixel) -> Pixel {
    let channel = |s: f32, d: f32| (s + d).min(1.0);

    Pixel {
        r: channel(src.r, dst.r),
        g: channel(src.g, dst.g),
        b: channel(src.b, dst.b),
        a: channel(src.a, dst.a),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Builds a pixel from its four channels.
    fn px(r: f32, g: f32, b: f32, a: f32) -> Pixel {
        Pixel { r, g, b, a }
    }

    /// The default blend mode is Normal.
    #[test]
    fn default_mode_is_normal() {
        assert_eq!(BlendMode::default(), BlendMode::Normal);
    }

    /// Normal mode gives the same result as `blend_normal`.
    #[test]
    fn normal_matches_blend_normal() {
        let src = px(0.5, 0.0, 0.0, 0.5);
        let dst = px(0.0, 0.0, 1.0, 1.0);
        assert_eq!(blend(BlendMode::Normal, src, dst), blend_normal(src, dst));
    }

    /// Multiplying with white changes nothing.
    #[test]
    fn multiply_with_white_is_identity() {
        let src = px(0.2, 0.4, 0.6, 1.0);
        let white = px(1.0, 1.0, 1.0, 1.0);
        assert_eq!(blend(BlendMode::Multiply, src, white), src);
    }

    /// Multiplying with black gives black.
    #[test]
    fn multiply_with_black_is_black() {
        let src = px(0.2, 0.4, 0.6, 1.0);
        let black = px(0.0, 0.0, 0.0, 1.0);
        assert_eq!(blend(BlendMode::Multiply, src, black), black);
    }

    /// Multiply multiplies the colour channels.
    #[test]
    fn multiply_multiplies_colors() {
        let src = px(0.5, 0.5, 0.5, 1.0);
        let dst = px(0.5, 1.0, 0.0, 1.0);
        assert_eq!(
            blend(BlendMode::Multiply, src, dst),
            px(0.25, 0.5, 0.0, 1.0)
        );
    }

    /// A transparent source leaves the destination unchanged.
    #[test]
    fn multiply_transparent_source_keeps_destination() {
        let dst = px(0.2, 0.4, 0.6, 1.0);
        assert_eq!(blend(BlendMode::Multiply, Pixel::default(), dst), dst);
    }

    /// Add sums the channels.
    #[test]
    fn add_sums_colors() {
        let src = px(0.25, 0.0, 0.5, 0.25);
        let dst = px(0.25, 0.5, 0.0, 0.5);
        assert_eq!(blend(BlendMode::Add, src, dst), px(0.5, 0.5, 0.5, 0.75));
    }

    /// Add caps every channel at 1.0.
    #[test]
    fn add_is_capped_at_one() {
        let src = px(0.75, 0.75, 0.75, 1.0);
        let dst = px(0.75, 0.0, 0.5, 1.0);
        assert_eq!(blend(BlendMode::Add, src, dst), px(1.0, 0.75, 1.0, 1.0));
    }

    /// Adding a transparent source changes nothing.
    #[test]
    fn add_transparent_keeps_destination() {
        let dst = px(0.2, 0.4, 0.6, 1.0);
        assert_eq!(blend(BlendMode::Add, Pixel::default(), dst), dst);
    }
}

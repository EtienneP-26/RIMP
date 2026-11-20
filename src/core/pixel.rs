//! RGBA pixel type and normal alpha blending.

/// An RGBA pixel with channels from 0.0 to 1.0.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Pixel {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

/// Blends `src` over `dst` using normal alpha compositing.
pub fn blend_normal(src: Pixel, dst: Pixel) -> Pixel {
    let k = 1.0 - src.a;

    Pixel {
        r: src.r + dst.r * k,
        g: src.g + dst.g * k,
        b: src.b + dst.b * k,
        a: src.a + dst.a * k,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Builds a pixel from its four channels.
    fn px(r: f32, g: f32, b: f32, a: f32) -> Pixel {
        Pixel { r, g, b, a }
    }

    /// An opaque source replaces the destination.
    #[test]
    fn opaque_source_replaces_destination() {
        let src = px(1.0, 0.0, 0.0, 1.0);
        let dst = px(0.0, 0.0, 1.0, 1.0);
        assert_eq!(blend_normal(src, dst), src);
    }

    /// A transparent source leaves the destination unchanged.
    #[test]
    fn transparent_source_keeps_destination() {
        let src = px(0.0, 0.0, 0.0, 0.0);
        let dst = px(0.2, 0.4, 0.6, 1.0);
        assert_eq!(blend_normal(src, dst), dst);
    }

    /// A half transparent source mixes with the destination.
    #[test]
    fn semi_transparent_source_mixes() {
        let src = px(0.5, 0.0, 0.0, 0.5);
        let dst = px(0.0, 0.0, 1.0, 1.0);
        assert_eq!(blend_normal(src, dst), px(0.5, 0.0, 0.5, 1.0));
    }

    /// The default pixel is fully transparent.
    #[test]
    fn default_pixel_is_transparent() {
        assert_eq!(Pixel::default(), px(0.0, 0.0, 0.0, 0.0));
    }
}

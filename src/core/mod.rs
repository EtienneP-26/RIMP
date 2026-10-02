#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Pixel {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

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

    fn px(r: f32, g: f32, b: f32, a: f32) -> Pixel {
        Pixel { r, g, b, a }
    }

    #[test]
    fn source_opaque_remplace_la_destination() {
        let src = px(1.0, 0.0, 0.0, 1.0);
        let dst = px(0.0, 0.0, 1.0, 1.0);
        assert_eq!(blend_normal(src, dst), src);
    }

    #[test]
    fn source_transparente_laisse_la_destination() {
        let src = px(0.0, 0.0, 0.0, 0.0);
        let dst = px(0.2, 0.4, 0.6, 1.0);
        assert_eq!(blend_normal(src, dst), dst);
    }

    #[test]
    fn source_semi_transparente_melange() {
        let src = px(0.5, 0.0, 0.0, 0.5);
        let dst = px(0.0, 0.0, 1.0, 1.0);
        assert_eq!(blend_normal(src, dst), px(0.5, 0.0, 0.5, 1.0));
    }
}
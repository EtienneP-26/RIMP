use super::pixel::{Pixel, blend_normal};

/// # How a source pixel is combined with the one below it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BlendMode {
    /// The source covers the destination according to its alpha.
    #[default]
    Normal,
    /// Colours are multiplied: the result is never lighter.
    Multiply,
    /// Colours are summed: the result is never darker.
    Add,
}

/// # Blends two premultiplied pixels with the given mode.
///
/// ## Arguments
/// * `mode` - The blend mode
/// * `src` - Upper pixel
/// * `dst` - Lower pixel
///
/// ## Returns
/// The blended pixel.
///
/// ## Examples
/// ```
/// use rimp::core::{BlendMode, Pixel, blend};
///
/// let gray = Pixel { r: 0.5, g: 0.5, b: 0.5, a: 1.0 };
/// let white = Pixel { r: 1.0, g: 1.0, b: 1.0, a: 1.0 };
/// assert_eq!(blend(BlendMode::Multiply, gray, white), gray);
/// ```
pub fn blend(mode: BlendMode, src: Pixel, dst: Pixel) -> Pixel {
    match mode {
        BlendMode::Normal => blend_normal(src, dst),
        BlendMode::Multiply => blend_multiply(src, dst),
        BlendMode::Add => blend_add(src, dst),
    }
}

/// # Multiply blend on premultiplied pixels.
///
/// Formula per colour channel: `src × (1 − dst.a) + dst × (1 − src.a) + src × dst`.
/// Alpha is composed as in the normal mode.
fn blend_multiply(src: Pixel, dst: Pixel) -> Pixel {
    let channel = |s: f32, d: f32| s * (1.0 - dst.a) + d * (1.0 - src.a) + s * d;

    Pixel {
        r: channel(src.r, dst.r),
        g: channel(src.g, dst.g),
        b: channel(src.b, dst.b),
        a: src.a + dst.a * (1.0 - src.a),
    }
}

/// # Additive blend on premultiplied pixels, each channel capped at 1.0.
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

    fn px(r: f32, g: f32, b: f32, a: f32) -> Pixel {
        Pixel { r, g, b, a }
    }

    #[test]
    fn mode_par_defaut_est_normal() {
        assert_eq!(BlendMode::default(), BlendMode::Normal);
    }

    #[test]
    fn normal_donne_le_meme_resultat_que_blend_normal() {
        let src = px(0.5, 0.0, 0.0, 0.5);
        let dst = px(0.0, 0.0, 1.0, 1.0);
        assert_eq!(blend(BlendMode::Normal, src, dst), blend_normal(src, dst));
    }

    #[test]
    fn produit_avec_blanc_ne_change_rien() {
        let src = px(0.2, 0.4, 0.6, 1.0);
        let blanc = px(1.0, 1.0, 1.0, 1.0);
        assert_eq!(blend(BlendMode::Multiply, src, blanc), src);
    }

    #[test]
    fn produit_avec_noir_donne_du_noir() {
        let src = px(0.2, 0.4, 0.6, 1.0);
        let noir = px(0.0, 0.0, 0.0, 1.0);
        assert_eq!(blend(BlendMode::Multiply, src, noir), noir);
    }

    #[test]
    fn produit_multiplie_les_couleurs() {
        let src = px(0.5, 0.5, 0.5, 1.0);
        let dst = px(0.5, 1.0, 0.0, 1.0);
        assert_eq!(blend(BlendMode::Multiply, src, dst), px(0.25, 0.5, 0.0, 1.0));
    }

    #[test]
    fn produit_source_transparente_laisse_la_destination() {
        let dst = px(0.2, 0.4, 0.6, 1.0);
        assert_eq!(blend(BlendMode::Multiply, Pixel::default(), dst), dst);
    }

    #[test]
    fn addition_additionne_les_couleurs() {
        let src = px(0.25, 0.0, 0.5, 0.25);
        let dst = px(0.25, 0.5, 0.0, 0.5);
        assert_eq!(blend(BlendMode::Add, src, dst), px(0.5, 0.5, 0.5, 0.75));
    }

    #[test]
    fn addition_est_plafonnee_a_un() {
        let src = px(0.75, 0.75, 0.75, 1.0);
        let dst = px(0.75, 0.0, 0.5, 1.0);
        assert_eq!(blend(BlendMode::Add, src, dst), px(1.0, 0.75, 1.0, 1.0));
    }

    #[test]
    fn addition_avec_transparent_ne_change_rien() {
        let dst = px(0.2, 0.4, 0.6, 1.0);
        assert_eq!(blend(BlendMode::Add, Pixel::default(), dst), dst);
    }
}


#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Colors {
    FRgba(f32, f32, f32, f32),
    FHsl(f32, f32, f32),
}

impl Colors {
    pub const BLACK: Self = Self::FRgba(0.0, 0.0, 0.0, 1.0);
    pub const WHITE: Self = Self::FRgba(1.0, 1.0, 1.0, 1.0);
    pub const RED: Self = Self::FRgba(1.0, 0.0, 0.0, 1.0);
    pub const GREEN: Self = Self::FRgba(0.0, 1.0, 0.0, 1.0);
    pub const BLUE: Self = Self::FRgba(0.0, 0.0, 1.0, 1.0);
    pub const TRANSPARENT: Self = Self::FRgba(0.0, 0.0, 0.0, 0.0);
}

impl From<[f32; 4]> for Colors {
    fn from(array: [f32; 4]) -> Self {
        Colors::FRgba(array[0], array[1], array[2], array[3])
    }
}

impl From<(f32, f32, f32, f32)> for Colors {
    fn from(tuple: (f32, f32, f32, f32)) -> Self {
        Colors::FRgba(tuple.0, tuple.1, tuple.2, tuple.3)
    }
}

impl From<Colors> for [f32; 4] {
    fn from(color: Colors) -> Self {
        match color {
            Colors::FRgba(r, g, b, a) => [r, g, b, a],
            Colors::FHsl(h, s, l) => {
                let rgb = Colors::hsl_to_rgb(h, s, l);
                [rgb.0, rgb.1, rgb.2, 1.0]
            }
        }
    }
}

impl From<Colors> for (f32, f32, f32, f32) {
    fn from(color: Colors) -> Self {
        match color {
            Colors::FRgba(r, g, b, a) => (r, g, b, a),
            Colors::FHsl(h, s, l) => {
                let rgb = Colors::hsl_to_rgb(h, s, l);
                (rgb.0, rgb.1, rgb.2, 1.0)
            }
        }
    }
}

impl Colors {
    fn hsl_to_rgb(hue: f32, saturation: f32, lightness: f32) -> (f32, f32, f32) {
        let c = lightness * saturation / 100.0;
        let x = c * (1.0 - (hue % 60.0).abs() / 60.0);
        let m = lightness - c;

        match hue {
            _ if (hue >= 0.0 && hue < 60.0) => (c, x, 0.0),
            _ if (hue >= 60.0 && hue < 120.0) => (x, c, 0.0),
            _ if (hue >= 120.0 && hue < 180.0) => (0.0, c, x),
            _ if (hue >= 180.0 && hue < 240.0) => (0.0, x, c),
            _ if (hue >= 240.0 && hue <= 300.0) => (x, 0.0, c),
            _ => ((c + m), (m - x), (m - x))
        }
    }
}
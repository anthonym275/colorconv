// sRGB <-> CIE L*a*b* (D65 white point, 2-degree observer).
//
// The path is always sRGB -> linear RGB -> CIE XYZ -> Lab and back. There's
// no shortcut between sRGB and Lab directly; XYZ is the common ground every
// reference (CIE, Bruce Lindbloom's tables) routes through, so that's what
// we use to check our own numbers against known values.

// D65 reference white, 2-degree observer.
const XN: f64 = 95.0489;
const YN: f64 = 100.0;
const ZN: f64 = 108.8840;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Lab {
    pub l: f64,
    pub a: f64,
    pub b: f64,
}

impl Rgb {
    pub fn from_hex(s: &str) -> Result<Rgb, String> {
        let s = s.trim().trim_start_matches('#');
        if s.len() != 6 {
            return Err(format!("expected 6 hex digits, got {:?}", s));
        }
        let byte = |i: usize| -> Result<u8, String> {
            u8::from_str_radix(&s[i..i + 2], 16)
                .map_err(|_| format!("invalid hex digits in {:?}", s))
        };
        Ok(Rgb {
            r: byte(0)?,
            g: byte(2)?,
            b: byte(4)?,
        })
    }

    pub fn to_hex(self) -> String {
        format!("{:02X}{:02X}{:02X}", self.r, self.g, self.b)
    }

    pub fn to_lab(self) -> Lab {
        let (x, y, z) = self.to_xyz();
        xyz_to_lab(x, y, z)
    }

    fn to_xyz(self) -> (f64, f64, f64) {
        let r = srgb_to_linear(self.r as f64 / 255.0);
        let g = srgb_to_linear(self.g as f64 / 255.0);
        let b = srgb_to_linear(self.b as f64 / 255.0);

        // sRGB -> XYZ, D65 (Bruce Lindbloom's matrix, scaled to 0..100).
        let x = 41.24564 * r + 35.75761 * g + 18.04375 * b;
        let y = 21.26729 * r + 71.51522 * g + 7.21750 * b;
        let z = 1.93339 * r + 11.91920 * g + 95.03041 * b;
        (x, y, z)
    }
}

impl Lab {
    pub fn to_rgb(self) -> Rgb {
        let (x, y, z) = lab_to_xyz(self);

        // XYZ -> sRGB, D65 (inverse of the matrix in Rgb::to_xyz).
        let x = x / 100.0;
        let y = y / 100.0;
        let z = z / 100.0;
        let r = 3.2404542 * x - 1.5371385 * y - 0.4985314 * z;
        let g = -0.9692660 * x + 1.8760108 * y + 0.0415560 * z;
        let b = 0.0556434 * x - 0.2040259 * y + 1.0572252 * z;

        Rgb {
            r: to_byte(linear_to_srgb(r)),
            g: to_byte(linear_to_srgb(g)),
            b: to_byte(linear_to_srgb(b)),
        }
    }
}

fn srgb_to_linear(c: f64) -> f64 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

fn linear_to_srgb(c: f64) -> f64 {
    let c = c.clamp(0.0, 1.0);
    if c <= 0.0031308 {
        c * 12.92
    } else {
        1.055 * c.powf(1.0 / 2.4) - 0.055
    }
}

// Round to the nearest byte instead of truncating; out-of-gamut Lab input
// (a caller can hand us any L/a/b triple) still clamps to a valid pixel
// rather than panicking.
fn to_byte(c: f64) -> u8 {
    (c * 255.0).round().clamp(0.0, 255.0) as u8
}

const DELTA: f64 = 6.0 / 29.0;

fn xyz_to_lab(x: f64, y: f64, z: f64) -> Lab {
    let fx = f_forward(x / XN);
    let fy = f_forward(y / YN);
    let fz = f_forward(z / ZN);
    Lab {
        l: 116.0 * fy - 16.0,
        a: 500.0 * (fx - fy),
        b: 200.0 * (fy - fz),
    }
}

fn lab_to_xyz(lab: Lab) -> (f64, f64, f64) {
    let fy = (lab.l + 16.0) / 116.0;
    let fx = fy + lab.a / 500.0;
    let fz = fy - lab.b / 200.0;
    (XN * f_inverse(fx), YN * f_inverse(fy), ZN * f_inverse(fz))
}

fn f_forward(t: f64) -> f64 {
    if t > DELTA.powi(3) {
        t.cbrt()
    } else {
        t / (3.0 * DELTA * DELTA) + 4.0 / 29.0
    }
}

fn f_inverse(t: f64) -> f64 {
    if t > DELTA {
        t * t * t
    } else {
        3.0 * DELTA * DELTA * (t - 4.0 / 29.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn white_is_lab_100() {
        let lab = Rgb { r: 255, g: 255, b: 255 }.to_lab();
        assert!((lab.l - 100.0).abs() < 0.05);
        assert!(lab.a.abs() < 0.05);
        assert!(lab.b.abs() < 0.05);
    }

    #[test]
    fn black_is_lab_0() {
        let lab = Rgb { r: 0, g: 0, b: 0 }.to_lab();
        assert!(lab.l.abs() < 0.05);
    }

    #[test]
    fn round_trip_holds_within_rounding() {
        let original = Rgb { r: 200, g: 80, b: 40 };
        let back = original.to_lab().to_rgb();
        assert!((original.r as i16 - back.r as i16).abs() <= 1);
        assert!((original.g as i16 - back.g as i16).abs() <= 1);
        assert!((original.b as i16 - back.b as i16).abs() <= 1);
    }

    #[test]
    fn hex_parsing_rejects_bad_input() {
        assert!(Rgb::from_hex("XYZ123").is_err());
        assert!(Rgb::from_hex("ABC").is_err());
        assert!(Rgb::from_hex("#336699").is_ok());
    }
}

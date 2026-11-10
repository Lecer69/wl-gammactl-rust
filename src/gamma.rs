/// Neutral color temperature in Kelvin (no tint applied).
pub const NEUTRAL_TEMPERATURE: f64 = 6500.0;
/// Supported color temperature range in Kelvin (limits of the Planckian locus approximation).
pub const MIN_TEMPERATURE: f64 = 1700.0;
pub const MAX_TEMPERATURE: f64 = 25000.0;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Params {
    pub contrast: f64,
    pub brightness: f64,
    pub gamma: f64,
    pub saturation: f64,
    /// Color temperature in Kelvin.
    pub temperature: f64,
}

impl Default for Params {
    fn default() -> Self {
        Self {
            contrast: 1.0,
            brightness: 1.0,
            gamma: 1.0,
            saturation: 1.0,
            temperature: NEUTRAL_TEMPERATURE,
        }
    }
}

pub fn fill_table(table: &mut [u16], ramp_size: u32, p: &Params) {
    let n = ramp_size as usize;
    let (r, rest) = table.split_at_mut(n);
    let (g, b) = rest.split_at_mut(n);

    const WR: f64 = 0.2126;
    const WG: f64 = 0.7152;
    const WB: f64 = 0.0722;

    let s = p.saturation;
    let [wp_r, wp_g, wp_b] = whitepoint(p.temperature);

    let scale_r = (WR * (1.0 - s) + s) * wp_r;
    let scale_g = (WG * (1.0 - s) + s) * wp_g;
    let scale_b = (WB * (1.0 - s) + s) * wp_b;

    for i in 0..n {
        let v = (i as f64 / (n - 1) as f64)
            .powf(1.0 / p.gamma)
            .mul_add(p.contrast, p.brightness - 1.0)
            .clamp(0.0, 1.0);

        r[i] = to_u16(v * scale_r);
        g[i] = to_u16(v * scale_g);
        b[i] = to_u16(v * scale_b);
    }
}

/// Per-channel multipliers (gamma-encoded, max channel = 1.0) that shift the
/// display white to the given blackbody color temperature. 6500 K yields exactly
/// `[1.0, 1.0, 1.0]`.
pub fn whitepoint(kelvin: f64) -> [f64; 3] {
    if kelvin == NEUTRAL_TEMPERATURE {
        return [1.0; 3];
    }

    let target = blackbody_linear_rgb(kelvin);
    let neutral = blackbody_linear_rgb(NEUTRAL_TEMPERATURE);

    let mut rgb = [0.0; 3];
    for c in 0..3 {
        rgb[c] = (target[c] / neutral[c]).max(0.0);
    }

    let max = rgb.iter().copied().fold(f64::MIN, f64::max);
    // Linear-light factors -> gamma-encoded factors for the ramp.
    rgb.map(|c| (c / max).powf(1.0 / 2.2))
}

/// Linear sRGB of a blackbody radiator at `kelvin`, with luminance Y = 1.
fn blackbody_linear_rgb(kelvin: f64) -> [f64; 3] {
    let (x, y) = planckian_xy(kelvin.clamp(MIN_TEMPERATURE, MAX_TEMPERATURE));

    // xyY -> XYZ
    let big_x = x / y;
    let big_y = 1.0;
    let big_z = (1.0 - x - y) / y;

    // XYZ -> linear sRGB (D65)
    [
        3.2404542 * big_x - 1.5371385 * big_y - 0.4985314 * big_z,
        -0.9692660 * big_x + 1.8760108 * big_y + 0.0415560 * big_z,
        0.0556434 * big_x - 0.2040259 * big_y + 1.0572252 * big_z,
    ]
}

/// CIE 1931 chromaticity of the Planckian locus (Kim et al., 2002),
/// valid for 1667 K..=25000 K.
fn planckian_xy(t: f64) -> (f64, f64) {
    let t2 = t * t;
    let t3 = t2 * t;

    let x = if t <= 4000.0 {
        -0.2661239e9 / t3 - 0.2343589e6 / t2 + 0.8776956e3 / t + 0.179910
    } else {
        -3.0258469e9 / t3 + 2.1070379e6 / t2 + 0.2226347e3 / t + 0.240390
    };

    let x2 = x * x;
    let x3 = x2 * x;

    let y = if t <= 2222.0 {
        -1.1063814 * x3 - 1.34811020 * x2 + 2.18555832 * x - 0.20219683
    } else if t <= 4000.0 {
        -0.9549476 * x3 - 1.37418593 * x2 + 2.09137015 * x - 0.16748867
    } else {
        3.0817580 * x3 - 5.87338670 * x2 + 3.75112997 * x - 0.37001483
    };

    (x, y)
}

#[inline]
fn to_u16(v: f64) -> u16 {
    (v.clamp(0.0, 1.0) * u16::MAX as f64) as u16
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn neutral_temperature_is_identity() {
        assert_eq!(whitepoint(NEUTRAL_TEMPERATURE), [1.0, 1.0, 1.0]);

        let n = 256;
        let mut table = vec![0u16; n * 3];
        fill_table(&mut table, n as u32, &Params::default());
        assert_eq!(table[n - 1], u16::MAX);
        assert_eq!(table[2 * n - 1], u16::MAX);
        assert_eq!(table[3 * n - 1], u16::MAX);
    }

    #[test]
    fn warm_temperatures_reduce_blue() {
        let mut prev_b = 1.0;
        for k in [6000.0, 5000.0, 4000.0, 3000.0, 2000.0, MIN_TEMPERATURE] {
            let [r, g, b] = whitepoint(k);
            assert!((r - 1.0).abs() < 1e-9, "{k}K: red should be max, got {r}");
            assert!(b < g && g < r, "{k}K: expected r > g > b, got {r} {g} {b}");
            assert!(b < prev_b, "{k}K: blue should decrease as temperature drops");
            prev_b = b;
        }
    }

    #[test]
    fn cool_temperatures_reduce_red() {
        for k in [8000.0, 10000.0, MAX_TEMPERATURE] {
            let [r, g, b] = whitepoint(k);
            assert!((b - 1.0).abs() < 1e-9, "{k}K: blue should be max, got {b}");
            assert!(r < g && g < b, "{k}K: expected r < g < b, got {r} {g} {b}");
        }
    }

    #[test]
    fn whitepoint_is_continuous() {
        // Compare in linear light: the 1/2.2 encoding is steep near zero, so the
        // encoded blue channel legitimately rises fast where it leaves the gamut (~1900 K).
        let linear = |k: f64| whitepoint(k).map(|c| c.powf(2.2));
        let mut k = MIN_TEMPERATURE;
        let mut prev = linear(k);
        while k < MAX_TEMPERATURE {
            k += 10.0;
            let cur = linear(k);
            for c in 0..3 {
                assert!((cur[c] - prev[c]).abs() < 0.01, "jump at {k}K: {prev:?} -> {cur:?}");
            }
            prev = cur;
        }
    }
}

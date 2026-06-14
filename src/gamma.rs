#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Params {
    pub contrast: f64,
    pub brightness: f64,
    pub gamma: f64,
    pub saturation: f64,
}

impl Default for Params {
    fn default() -> Self {
        Self {
            contrast: 1.0,
            brightness: 1.0,
            gamma: 1.0,
            saturation: 1.0,
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

    let scale_r = WR * (1.0 - s) + s;
    let scale_g = WG * (1.0 - s) + s;
    let scale_b = WB * (1.0 - s) + s;

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

#[inline]
fn to_u16(v: f64) -> u16 {
    (v.clamp(0.0, 1.0) * u16::MAX as f64) as u16
}

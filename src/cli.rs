use clap::Parser;

use crate::gamma::{Params, MAX_TEMPERATURE, MIN_TEMPERATURE, NEUTRAL_TEMPERATURE};

#[derive(Parser, Debug)]
#[command(
    name = "wl-gammactl-rust",
    about = "Change contrast, brightness, gamma, saturation & color temperature on Wayland"
)]
pub struct Args {
    #[arg(short, long, value_name = "FLOAT")]
    pub contrast: Option<f64>,

    #[arg(short, long, value_name = "FLOAT")]
    pub brightness: Option<f64>,

    #[arg(short, long, value_name = "FLOAT")]
    pub gamma: Option<f64>,

    #[arg(short, long, value_name = "FLOAT")]
    pub saturation: Option<f64>,

    /// Color temperature in Kelvin (1700-25000, neutral: 6500)
    #[arg(short, long, value_name = "KELVIN", value_parser = parse_temperature)]
    pub temperature: Option<f64>,

    #[arg(long)]
    pub gui: bool,
}

impl Args {
    pub fn has_values(&self) -> bool {
        self.contrast.is_some()
            || self.brightness.is_some()
            || self.gamma.is_some()
            || self.saturation.is_some()
            || self.temperature.is_some()
    }

    pub fn params(&self) -> Params {
        Params {
            contrast: self.contrast.unwrap_or(1.0),
            brightness: self.brightness.unwrap_or(1.0),
            gamma: self.gamma.unwrap_or(1.0),
            saturation: self.saturation.unwrap_or(1.0),
            temperature: self.temperature.unwrap_or(NEUTRAL_TEMPERATURE),
        }
    }
}

fn parse_temperature(s: &str) -> Result<f64, String> {
    let k: f64 = s.parse().map_err(|_| format!("`{s}` is not a number"))?;
    if (MIN_TEMPERATURE..=MAX_TEMPERATURE).contains(&k) {
        Ok(k)
    } else {
        Err(format!("must be between {MIN_TEMPERATURE} and {MAX_TEMPERATURE} Kelvin"))
    }
}

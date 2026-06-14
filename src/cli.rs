use clap::Parser;

#[derive(Parser, Debug)]
#[command(
    name = "wl-gammactl-rust",
    about = "Change contrast, brightness, gamma & saturation on Wayland"
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

    #[arg(long)]
    pub gui: bool,
}

impl Args {
    pub fn has_values(&self) -> bool {
        self.contrast.is_some() || self.brightness.is_some() || self.gamma.is_some() || self.saturation.is_some()
    }
}

mod cli;
mod gamma;
mod gui;
mod wayland;

use anyhow::Result;
use clap::Parser;
use gamma::Params;
use std::sync::{Arc, Mutex};

fn main() -> Result<()> {
    let args = cli::Args::parse();

    let (mut wl, _conn, mut queue) = wayland::WaylandState::connect()?;

    if args.has_values() && !args.gui {
        // cli mode
        let params = Params {
            contrast:   args.contrast.unwrap_or(1.0),
            brightness: args.brightness.unwrap_or(1.0),
            gamma:      args.gamma.unwrap_or(1.0),
            saturation: args.saturation.unwrap_or(1.0),
        };
        wl.apply(&params, &mut queue)?;

        // Keep connection alive
        loop {
            queue.roundtrip(&mut wl)?;
            std::thread::sleep(std::time::Duration::from_secs(1));
        }
    }

    let initial = Params {
        contrast: args.contrast.unwrap_or(1.0),
        brightness: args.brightness.unwrap_or(1.0),
        gamma: args.gamma.unwrap_or(1.0),
        saturation: args.saturation.unwrap_or(1.0),
    };

    let wl = Arc::new(Mutex::new(wl));
    let queue = Arc::new(Mutex::new(queue));

    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_title("wl-gammactl-rust")
            .with_inner_size([420.0, 240.0])
            .with_resizable(false),
        ..Default::default()
    };

    eframe::run_native(
        "wl-gammactl-rust",
        options,
        Box::new(move |cc| {
            let wl = Arc::clone(&wl);
            let queue = Arc::clone(&queue);

            Ok(Box::new(gui::App::new(cc, initial, move |params| {
                if let (Ok(mut wl), Ok(mut q)) = (wl.lock(), queue.lock()) {
                    let _ = wl.apply(params, &mut q);
                }
            })))
        }),
    )
    .map_err(|e| anyhow::anyhow!("eframe: {e}"))?;

    Ok(())
}

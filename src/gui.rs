use eframe::egui::{self, Color32, Margin, RichText, Slider, Stroke, Vec2};

use crate::gamma::Params;

pub struct App {
    params: Params,
    on_change: Box<dyn Fn(&Params) + Send>,
}

impl App {
    pub fn new(
        cc: &eframe::CreationContext,
        initial: Params,
        on_change: impl Fn(&Params) + Send + 'static,
    ) -> Self {
        apply_style(&cc.egui_ctx);
        Self {
            params: initial,
            on_change: Box::new(on_change),
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let frame = egui::Frame::central_panel(&ctx.style()).inner_margin(Margin::same(20.0));

        egui::CentralPanel::default().frame(frame).show(ctx, |ui| {
            ui.heading("wl-gammactl-rust");
            ui.add_space(12.0);

            let before = self.params;

            slider(ui, "Contrast", &mut self.params.contrast, 0.0, 2.0);
            slider(ui, "Brightness", &mut self.params.brightness, 0.0, 2.0);
            slider(ui, "Gamma", &mut self.params.gamma, 0.1, 5.0);
            slider(ui, "Saturation", &mut self.params.saturation, 0.0, 2.0);

            ui.add_space(12.0);

            let cmd = format!(
                "wl-gammactl-rust -c {:.3} -b {:.3} -g {:.3} -s {:.3}",
                self.params.contrast,
                self.params.brightness,
                self.params.gamma,
                self.params.saturation,
            );
            ui.label(
                RichText::new(&cmd)
                    .monospace()
                    .color(Color32::from_gray(130)),
            );

            ui.add_space(12.0);

            if ui.button("Reset").clicked() {
                self.params = Params::default();
            }

            if self.params != before {
                (self.on_change)(&self.params);
            }
        });
    }
}

fn slider(ui: &mut egui::Ui, label: &str, value: &mut f64, min: f64, max: f64) {
    ui.horizontal(|ui| {
        ui.label(RichText::new(format!("{label:<12}")).monospace());
        ui.add(
            Slider::new(value, min..=max)
                .step_by(0.001)
                .fixed_decimals(3),
        );
    });
    ui.add_space(4.0);
}

fn apply_style(ctx: &egui::Context) {
    ctx.style_mut(|style| {
        style.visuals = egui::Visuals::dark();
        style.visuals.window_fill = Color32::from_rgb(15, 15, 15);
        style.visuals.panel_fill = Color32::from_rgb(15, 15, 15);
        style.visuals.widgets.inactive.bg_fill = Color32::from_rgb(30, 30, 30);
        style.visuals.widgets.hovered.bg_fill = Color32::from_rgb(45, 45, 45);
        style.visuals.widgets.active.bg_fill = Color32::from_rgb(60, 60, 60);
        style.visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, Color32::from_gray(180));
        style.visuals.selection.bg_fill = Color32::from_rgb(80, 120, 200);
        style.spacing.slider_width = 260.0;
        style.spacing.item_spacing = Vec2::new(8.0, 6.0);
    });
}

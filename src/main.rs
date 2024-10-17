use anyhow::Context;
use eframe::NativeOptions;
use egui::ViewportBuilder;
use log::LevelFilter;
use simplelog::{ColorChoice, TerminalMode};

mod app;

fn main() -> anyhow::Result<()> {
    simplelog::TermLogger::init(
        LevelFilter::Debug,
        simplelog::Config::default(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )
    .context("Failed to initialize logger")?;

    eframe::run_native(
        "Pulse Spectralyzer",
        NativeOptions {
            viewport: ViewportBuilder::default().with_inner_size(egui::vec2(1600.0, 700.0)),
            ..Default::default()
        },
        Box::new(|cc| Ok(Box::new(app::App::new(cc)))),
    )
    .map_err(|e| anyhow::anyhow!("Failed to run eframe app: {e}"))?;

    Ok(())
}

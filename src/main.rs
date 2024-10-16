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

#[derive(Debug, Clone, Copy)]
struct LogScale {
    base: f64,
}

impl LogScale {
    pub const fn new(base: f64) -> Self {
        Self { base }
    }

    /// map a value from [0, 1] by the logaritmic scale in [0, 1]
    pub fn map(self, x: f64) -> f64 {
        (self.base.powf(x) - 1.0) / (self.base - 1.0)
    }

    /// reverse map a value from [0, 1] by the logaritmic scale in [0, 1]
    pub fn map_inv(self, x: f64) -> f64 {
        (self.base - 1.0).mul_add(x, 1.0).log(self.base)
    }
}

trait Lerp<T> {
    fn lerp(self, other: Self, t: T) -> Self;
}

impl Lerp<Self> for f32 {
    fn lerp(self, other: Self, t: Self) -> Self {
        self.mul_add(1.0 - t, other * t)
    }
}

trait Deinterleave<T: Iterator<Item = I>, I> {
    fn deinterleave(self, channels: usize) -> Vec<Vec<I>>;
}

impl<T: Iterator<Item = I>, I> Deinterleave<T, I> for T {
    fn deinterleave(self, n: usize) -> Vec<Vec<I>> {
        let mut channels = (0..n).map(|_| Vec::new()).collect::<Vec<_>>();

        for (i, sample) in self.enumerate() {
            channels[i % n].push(sample);
        }

        channels
    }
}

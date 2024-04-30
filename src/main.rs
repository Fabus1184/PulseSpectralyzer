use std::sync::Arc;

use anyhow::Context;
use cpal::traits::{DeviceTrait, HostTrait};
use eframe::{CreationContext, NativeOptions};
use egui::{ecolor::rgb_from_hsv, mutex::Mutex, Color32, Frame, ViewportBuilder};
use egui_plot::{Bar, BarChart, GridMark, Plot};
use log::{error, LevelFilter};
use ringbuffer::AllocRingBuffer;
use ringbuffer::RingBuffer;
use rustfft::{num_complex::Complex, FftPlanner};
use simplelog::{ColorChoice, TerminalMode};

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
        Box::new(|cc| Box::new(App::new(cc).expect("Failed to build app"))),
    )
    .map_err(|e| anyhow::anyhow!("Failed to run eframe app: {e}"))?;

    Ok(())
}

struct App {
    stream: Option<(cpal::Stream, cpal::StreamConfig, String)>,
    log_scale: LogScale,
    data: Arc<Mutex<AllocRingBuffer<f32>>>,
    devices: Vec<cpal::Device>,
    device: Option<cpal::Device>,
}

impl App {
    pub fn new(_creation_context: &CreationContext) -> anyhow::Result<Self> {
        let this = Self {
            stream: None,
            log_scale: LogScale::new(5.0),
            data: Arc::new(Mutex::new(AllocRingBuffer::new(1))),
            devices: cpal::default_host().devices().unwrap().collect(),
            device: None,
        };

        Ok(this)
    }
}

fn blackman_harris<T: Iterator<Item = f32>>(n: usize, iterator: T) -> impl Iterator<Item = f32> {
    let a = [0.35875, 0.48829, 0.14128, 0.01168];
    let n = n as f32;

    iterator.enumerate().map(move |(i, x)| {
        let i = i as f32;
        x * (a[0] - a[1] * (2.0 * std::f32::consts::PI * i / n).cos()
            + a[2] * (4.0 * std::f32::consts::PI * i / n).cos()
            - a[3] * (6.0 * std::f32::consts::PI * i / n).cos())
    })
}

#[derive(Debug, Clone, Copy)]
struct LogScale {
    base: f64,
}

impl LogScale {
    pub fn new(base: f64) -> Self {
        Self { base }
    }

    /// map a value from [0, 1] by the logaritmic scale in [0, 1]
    pub fn map(&self, x: f64) -> f64 {
        (self.base.powf(x) - 1.0) / (self.base - 1.0)
    }

    /// reverse map a value from [0, 1] by the logaritmic scale in [0, 1]
    pub fn map_inv(&self, x: f64) -> f64 {
        ((self.base - 1.0) * x + 1.0).log(self.base)
    }
}

impl App {
    fn format_y(&self, y: f64) -> String {
        format!("{:.02} dB", y * 10.0)
    }

    fn format_x(&self, x: f64) -> String {
        let sample_rate = self
            .stream
            .as_ref()
            .map(|(_, config, _)| config.sample_rate.0)
            .unwrap_or(0);

        format!(
            "{:.02} kHz",
            self.log_scale.map(x) * sample_rate as f64 / 2.0 / 1_000.0
        )
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal_top(|ui| {
                Frame::group(ui.style()).show(ui, |ui| {
                    let sample_rate = self
                        .stream
                        .as_ref()
                        .map(|(_, config, _)| config.sample_rate.0)
                        .unwrap_or(0);

                    Plot::new("spectrum")
                        .width(ui.available_width() * 0.8)
                        .include_y(3)
                        .include_y(-3)
                        .include_x(1.0)
                        .legend(egui_plot::Legend::default())
                        .x_grid_spacer(|_| {
                            (0..=sample_rate / 2)
                                .step_by(2000)
                                .map(|f| GridMark {
                                    value: self
                                        .log_scale
                                        .map_inv(f as f64 / sample_rate as f64 * 2.0),
                                    step_size: self
                                        .log_scale
                                        .map_inv(2000.0 / sample_rate as f64 * 2.0),
                                })
                                .collect()
                        })
                        .y_axis_formatter(|mark, _, _| self.format_y(mark.value))
                        .x_axis_formatter(|mark, _, _| self.format_x(mark.value))
                        .label_formatter(|_, &egui_plot::PlotPoint { x, y }| {
                            format!("{}\n{}", self.format_x(x), self.format_y(y))
                        })
                        .show(ui, |plot| {
                            let data = self.data.lock();

                            let channels = match self.stream.as_ref() {
                                Some((_, config, _)) if config.channels == 1 => 1,
                                Some((_, config, _)) if config.channels == 2 => 2,
                                _ => 1,
                            };

                            let channels = data.iter().copied().deinterleave(channels);

                            drop(data);

                            for (channel, data) in channels.into_iter().enumerate() {
                                let mut planner = FftPlanner::new();
                                let fft = planner.plan_fft_forward(data.len());

                                let mut buffer = blackman_harris(data.len(), data.iter().copied())
                                    .map(|x| Complex::new(x, 0.0))
                                    .collect::<Vec<_>>();

                                fft.process(&mut buffer);

                                buffer.truncate(buffer.len() / 2);

                                let buffer = buffer
                                    .iter()
                                    .map(|x| (x.re.abs() + 1.0).log10())
                                    .collect::<Vec<_>>();

                                let bars = (0..buffer.len().saturating_sub(1))
                                    .map(|i| {
                                        let x = self.log_scale.map(i as f64 / buffer.len() as f64);
                                        let y = buffer[(x * buffer.len() as f64) as usize].lerp(
                                            buffer[(x * buffer.len() as f64 + 1.0) as usize],
                                            x.fract() as f32,
                                        );

                                        // exponential scale
                                        let y = ((y / 3.0 + 1.0).powf(5f32) - 1.0) / 7.0;

                                        let [r, g, b] = rgb_from_hsv((0.7 + y / 3.0, 1.0, 1.0));
                                        Bar::new(
                                            i as f64 / buffer.len() as f64,
                                            y as f64 * -((channel % 2) as f64 - 0.5).signum(),
                                        )
                                        .fill(
                                            Color32::from_rgb(
                                                (r * 255.0) as u8,
                                                (g * 255.0) as u8,
                                                (b * 255.0) as u8,
                                            ),
                                        )
                                    })
                                    .collect::<Vec<_>>();

                                plot.bar_chart(
                                    BarChart::new(bars)
                                        .name(format!("Channel {}", channel + 1))
                                        .vertical()
                                        .width(0.9 / buffer.len() as f64),
                                );
                            }
                        });
                });

                Frame::group(ui.style()).show(ui, |ui| {
                    ui.vertical(|ui| {
                        ui.heading("Stream settings");

                        egui::ComboBox::new("devices", "Device")
                            .selected_text(
                                self.device
                                    .as_ref()
                                    .map(|device| device.name().unwrap())
                                    .unwrap_or(String::from("None")),
                            )
                            .show_ui(ui, |ui| {
                                for device in self.devices.iter() {
                                    if ui.selectable_label(false, device.name().unwrap()).clicked()
                                    {
                                        self.device = Some(device.clone());
                                    }
                                }
                            });

                        match (self.stream.as_mut(), self.device.as_mut()) {
                            (Some((_, config, ref name)), _) => {
                                ui.label(format!("Streaming from: {}", name));

                                ui.label(format!("Sample rate: {} Hz", config.sample_rate.0));
                                ui.label(format!("Channels: {}", config.channels));
                                ui.label(format!("Buffer size: {:?}", config.buffer_size));

                                if ui.button("Stop").clicked() {
                                    self.stream = None;
                                    self.data.lock().clear();
                                }
                            }
                            (None, Some(ref device)) => {
                                if ui.button("Start").clicked() {
                                    let data = self.data.clone();

                                    let config = cpal::StreamConfig {
                                        // channels: 1,
                                        ..device.default_input_config().unwrap().config()
                                    };

                                    self.stream = Some((
                                        device
                                            .build_input_stream(
                                                &config,
                                                move |samples: &[f32], _| {
                                                    data.lock().extend(samples.iter().copied());
                                                },
                                                |error| {
                                                    error!("Error in audio stream: {:?}", error);
                                                },
                                                None,
                                            )
                                            .unwrap(),
                                        config.clone(),
                                        device.name().unwrap().to_string(),
                                    ));

                                    *self.data.lock() = AllocRingBuffer::new(
                                        config.sample_rate.0 as usize / 5
                                            * config.channels as usize,
                                    );
                                }
                            }
                            (None, None) => {
                                ui.add_enabled(false, egui::Button::new("Start"));
                            }
                        }

                        ui.heading("Display settings");

                        ui.horizontal(|ui| {
                            ui.label("Log scale base");
                            ui.add(
                                egui::DragValue::new(&mut self.log_scale.base)
                                    .speed(0.1)
                                    .clamp_range(2.0..=40.0),
                            );
                        });
                    });
                });
            });
        });

        ctx.request_repaint();
    }
}

trait Lerp<T> {
    fn lerp(self, other: Self, t: T) -> Self;
}

impl Lerp<f32> for f32 {
    fn lerp(self, other: Self, t: f32) -> Self {
        self * (1.0 - t) + other * t
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

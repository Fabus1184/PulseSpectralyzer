use std::sync::Arc;

use cpal::traits::{DeviceTrait, HostTrait};
use eframe::CreationContext;
use egui::RichText;
use egui::{mutex::Mutex, Frame};
use egui_plot::{Bar, BarChart, Plot};
use ringbuffer::AllocRingBuffer;
use ringbuffer::RingBuffer;
use rustfft::{num_complex::Complex, FftPlanner};

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

#[allow(clippy::suboptimal_flops)] // Readability is more important
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

enum State {
    Stopped,
    Selected {
        device: cpal::Device,
        config_ranges: Vec<cpal::SupportedStreamConfigRange>,
        stream_config: cpal::StreamConfig,
    },
    Streaming {
        device: cpal::Device,
        config_ranges: Vec<cpal::SupportedStreamConfigRange>,
        stream_config: cpal::StreamConfig,
        data: Arc<Mutex<AllocRingBuffer<f32>>>,
        stream: cpal::Stream,
    },
}

pub struct App {
    devices: Vec<cpal::Device>,
    state: State,
}

impl App {
    pub fn new(cc: &CreationContext) -> Self {
        let hosts = cpal::available_hosts();
        log::info!("Available audio hosts: {hosts:?}");

        cc.egui_ctx.all_styles_mut(|style| {
            style.spacing.item_spacing *= 2.0;
            style.spacing.button_padding *= 2.0;
        });

        Self {
            devices: hosts
                .iter()
                .filter_map(|host| cpal::host_from_id(*host).ok())
                .filter_map(|host| host.devices().ok())
                .flatten()
                .collect(),
            state: State::Stopped,
        }
    }
}

struct XScale;
impl XScale {
    fn f_to_x(frequency: f64) -> f64 {
        if frequency <= 200.0 {
            frequency * 14.0
        } else if frequency <= 20_000.0 {
            (20_000f64.powi(2) - (frequency - 20_000.0).powi(2)).sqrt()
        } else {
            frequency
        }
    }

    fn x_to_f(x: f64) -> f64 {
        if x <= 200.0 {
            x / 14.0
        } else if x <= 20_000.0 {
            20_000.0 - (20_000f64.powi(2) - x.powi(2)).sqrt()
        } else {
            x
        }
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE.inner_margin(10.0))
            .show_inside(ui, |ui| {
                ui.horizontal_top(|ui| {
                    Plot::new("spectrum")
                        .width(ui.available_width() * 0.75)
                        .include_y(100)
                        .include_y(-100)
                        .x_axis_label("Frequency")
                        .y_axis_label("Amplitude (dBFS = dB Full Scale)")
                        .legend(egui_plot::Legend::default())
                        .x_axis_formatter(|mark, _| {
                            format!("{:.01} kHz", XScale::x_to_f(mark.value) / 1000.0)
                        })
                        .y_axis_formatter(|mark, _| {
                            format!("{:.01} dB", (mark.value.abs() - 100.0))
                        })
                        .x_grid_spacer(|grid| egui_plot::log_grid_spacer(5)(grid))
                        .show(ui, |plot| {
                            let State::Streaming {
                                ref data,
                                ref stream_config,
                                ..
                            } = self.state
                            else {
                                return;
                            };

                            let channels = { data.lock().iter().copied() }
                                .deinterleave(stream_config.channels as usize);

                            for (channel, data) in channels.into_iter().enumerate() {
                                let mut planner = FftPlanner::new();
                                let fft = planner.plan_fft_forward(data.len());

                                let mut buffer = blackman_harris(data.len(), data.iter().copied())
                                    .map(|x| Complex::new(x, 0.0))
                                    .collect::<Vec<_>>();

                                fft.process(&mut buffer);

                                buffer.truncate(buffer.len() / 2);

                                let len = buffer.len() as f64;
                                let bars = buffer
                                    .iter()
                                    .enumerate()
                                    .map(|(i, &iq)| {
                                        let frequency =
                                            i as f64 / len * stream_config.sample_rate as f64 / 2.0;

                                        let amplitude = (iq.norm() as f64) / len;
                                        let decibel = 20.0 * amplitude.log10();
                                        let y = decibel.clamp(-100.0, 0.0) + 100.0;

                                        let [r, g, b] = egui::ecolor::rgb_from_hsv((
                                            ((y as f32 / 100.0) + 0.3) % 1.0,
                                            1.0,
                                            1.0,
                                        ))
                                        .map(|c| (c * 255.0).round() as u8);

                                        let x0 = XScale::f_to_x(frequency);
                                        let x1 = XScale::f_to_x(
                                            (i + 1) as f64 / len * stream_config.sample_rate as f64
                                                / 2.0,
                                        );

                                        Bar::new(
                                            (x0 + x1) / 2.0,
                                            if channel % 2 == 0 { y } else { -y },
                                        )
                                        .fill(egui::Color32::from_rgb(r, g, b))
                                        .width(x1 - x0)
                                    })
                                    .collect::<Vec<_>>();

                                plot.bar_chart(
                                    BarChart::new(format!("Channel {}", channel + 1), bars)
                                        .vertical()
                                        .element_formatter(Box::new(|bar, _| {
                                            format!(
                                                "{:.01} kHz: {:.01} dB",
                                                XScale::x_to_f(bar.argument) / 1000.0,
                                                bar.value.abs() - 100.0
                                            )
                                        })),
                                );
                            }
                        });

                    Frame::group(ui.style()).show(ui, |ui| {
                        ui.take_available_width();

                        ui.vertical(|ui| {
                            ui.heading("Device");

                            egui::ComboBox::new("devices", "")
                                .selected_text(match self.state {
                                    State::Stopped => String::from("None"),
                                    State::Selected { ref device, .. }
                                    | State::Streaming { ref device, .. } => device.to_string(),
                                })
                                .show_ui(ui, |ui| {
                                    for device in &self.devices {
                                        if ui.selectable_label(false, device.to_string()).clicked()
                                        {
                                            let config_ranges = device
                                                .supported_input_configs()
                                                .expect("Could not get supported input configs")
                                                .collect::<Vec<_>>();

                                            self.state = State::Selected {
                                                device: device.clone(),
                                                config_ranges,
                                                stream_config: device
                                                    .default_input_config()
                                                    .expect("Could not get default input config")
                                                    .config(),
                                            };
                                        }
                                    }
                                });

                            ui.add_space(10.0);

                            ui.heading("Supported Configs");
                            ui.label(
                                RichText::new(
                                    "Usually lower sample rates also work even if not listed here",
                                )
                                .size(10.0),
                            );

                            if let State::Selected {
                                ref config_ranges, ..
                            }
                            | State::Streaming {
                                ref config_ranges, ..
                            } = self.state
                            {
                                for &config_range in config_ranges {
                                    ui.label(
                                        RichText::new(format!(
                                            "· {:>3} {}-{} kHz, {} channels, {} samples",
                                            config_range.sample_format(),
                                            config_range.min_sample_rate() as f32 / 1000.0,
                                            config_range.max_sample_rate() as f32 / 1000.0,
                                            config_range.channels(),
                                            match config_range.buffer_size() {
                                                cpal::SupportedBufferSize::Range { min, max } =>
                                                    format!("{min}-{max}"),
                                                cpal::SupportedBufferSize::Unknown =>
                                                    String::from("?"),
                                            }
                                        ))
                                        .monospace()
                                        .size(10.0),
                                    );
                                }
                            }

                            ui.add_space(10.0);

                            ui.heading("Stream Config");

                            // sample rate drag value
                            match self.state {
                                State::Stopped => {
                                    ui.add_enabled(
                                        false,
                                        egui::DragValue::new(&mut 0).prefix("Sample rate: "),
                                    );
                                }
                                State::Selected {
                                    ref mut stream_config,
                                    ..
                                } => {
                                    ui.add(
                                        egui::DragValue::new(&mut stream_config.sample_rate)
                                            .prefix("Sample rate: "),
                                    );
                                }
                                State::Streaming {
                                    ref mut stream_config,
                                    ..
                                } => {
                                    ui.add_enabled(
                                        false,
                                        egui::DragValue::new(&mut stream_config.sample_rate)
                                            .prefix("Sample rate: "),
                                    );
                                }
                            }

                            ui.add_space(10.0);

                            match self.state {
                                State::Stopped => {
                                    ui.add_enabled(false, egui::Button::new("▶ Start"));
                                }
                                State::Selected {
                                    ref device,
                                    ref config_ranges,
                                    stream_config,
                                } => {
                                    if ui.button("▶ Start").clicked() {
                                        let data = Arc::new(Mutex::new(AllocRingBuffer::new(
                                            stream_config.sample_rate as usize / 5
                                                * stream_config.channels as usize,
                                        )));

                                        let data_ = data.clone();
                                        let stream = device
                                            .build_input_stream(
                                                stream_config,
                                                move |samples: &[f32], _| {
                                                    data_.lock().extend(samples.iter().copied());
                                                },
                                                |error| {
                                                    log::error!("Error in audio stream: {error:?}");
                                                },
                                                None,
                                            )
                                            .expect("Could not build input stream");

                                        self.state = State::Streaming {
                                            device: device.clone(),
                                            config_ranges: config_ranges.clone(),
                                            stream_config,
                                            data: data.clone(),
                                            stream,
                                        };
                                    }
                                }
                                State::Streaming {
                                    ref device,
                                    ref config_ranges,
                                    stream_config,
                                    ..
                                } => {
                                    if ui.button("⏹ Stop").clicked() {
                                        self.state = State::Selected {
                                            device: device.clone(),
                                            config_ranges: config_ranges.clone(),
                                            stream_config,
                                        };
                                    }
                                }
                            }
                        });
                    });
                });
            });

        ui.request_repaint_after(std::time::Duration::from_millis(10));
    }
}

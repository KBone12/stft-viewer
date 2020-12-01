use std::f32::consts::PI;

use plotters::{
    chart::ChartBuilder,
    drawing::IntoDrawingArea,
    element::Rectangle,
    series::LineSeries,
    style::{
        colors::{BLUE, GREEN, WHITE},
        Color, ShapeStyle,
    },
};
use plotters_canvas::CanvasBackend;

use rustfft::{num_complex::Complex, num_traits::Zero, FFTplanner};

use wasm_bindgen::prelude::wasm_bindgen;

use web_sys::HtmlCanvasElement;

#[wasm_bindgen]
pub enum WindowFunction {
    Blackman,
    Hamming,
    Hann,
    Rectangle,
}

impl WindowFunction {
    pub fn generate(&self, length: usize) -> Vec<f32> {
        match self {
            WindowFunction::Blackman => (0..length)
                .map(|i| {
                    0.42 - 0.5 * (2.0 * PI * i as f32).cos()
                        + 0.08 * (4.0 * PI * i as f32 / length as f32).cos()
                })
                .collect(),
            WindowFunction::Hamming => (0..length)
                .map(|i| 0.54 - 0.46 * (2.0 * PI * i as f32 / length as f32).cos())
                .collect(),
            WindowFunction::Hann => (0..length)
                .map(|i| 0.5 - 0.5 * (2.0 * PI * i as f32 / length as f32).cos())
                .collect(),
            WindowFunction::Rectangle => {
                vec![1.0; length]
            }
        }
    }
}

#[wasm_bindgen]
pub struct FourierViewer {
    audio_data: Vec<f32>,
    sample_rate: f32,
    fft_planner: FFTplanner<f32>,
    spectra: Option<Vec<Complex<f32>>>,
}

#[wasm_bindgen]
impl FourierViewer {
    #[wasm_bindgen(constructor)]
    pub fn new(audio_data: &[f32], sample_rate: f32) -> Self {
        Self {
            audio_data: audio_data.to_vec(),
            sample_rate,
            fft_planner: FFTplanner::new(false),
            spectra: None,
        }
    }

    pub fn run_fft(&mut self, size: usize, window_function: WindowFunction) {
        let fft = self.fft_planner.plan_fft(size);

        let window = window_function.generate(size.min(self.audio_data.len()));
        let mut input: Vec<_> = self
            .audio_data
            .iter()
            .take(size.min(self.audio_data.len()))
            .zip(window.iter())
            .map(|(x, w)| x * w)
            .collect();
        input.resize(size, 0.0);
        input.rotate_right((size - size.min(self.audio_data.len())) / 2);
        let mut input: Vec<_> = input.iter().map(|f| Complex::new(*f, 0.0)).collect();
        let mut output = vec![Complex::zero(); size];
        fft.process(&mut input, &mut output);
        self.spectra = Some(output);
    }

    pub fn peak_frequencies(&self, num: usize) -> Option<Vec<f32>> {
        self.spectra.as_ref().and_then(|spectra| {
            let df = self.sample_rate / spectra.len() as f32;
            let spectra: Vec<_> = spectra.iter().take(spectra.len() / 2).collect();
            let num = num.min(spectra.len());
            let mut tmp: Vec<_> = spectra.iter().map(|c| c.norm_sqr()).enumerate().collect();
            tmp.sort_by(|(_, a), (_, b)| b.partial_cmp(a).expect("Contains NaN"));
            Some(tmp[..num].iter().map(|(i, _)| *i as f32 * df).collect())
        })
    }

    pub fn peak_phases(&self, num: usize) -> Option<Vec<f32>> {
        self.spectra.as_ref().and_then(|spectra| {
            let spectra: Vec<_> = spectra.iter().take(spectra.len() / 2).collect();
            let num = num.min(spectra.len());
            let mut tmp: Vec<_> = spectra.iter().enumerate().collect();
            tmp.sort_by(|(_, a), (_, b)| {
                (b.norm_sqr())
                    .partial_cmp(&a.norm_sqr())
                    .expect("Contains NaN")
            });
            let mut phases: Vec<_> = tmp[..num].iter().map(|(_, s)| s.arg()).collect();
            unwrap_phase(&mut phases);
            Some(phases)
        })
    }

    pub fn draw(&self, canvas: HtmlCanvasElement) {
        let root = CanvasBackend::with_canvas_object(canvas)
            .expect("Illegal canvas")
            .into_drawing_area();
        root.fill(&WHITE)
            .expect("Some errors have been occurred in the backend");
        let areas = root.split_evenly((2, 1));
        let time_area = &areas[0];
        let frequency_area = &areas[1];

        let audio_range = plotters::data::fitting_range(&self.audio_data);
        let mut chart = ChartBuilder::on(&time_area)
            .x_label_area_size(50)
            .y_label_area_size(60)
            .build_cartesian_2d(0..self.audio_data.len(), audio_range.clone())
            .expect("Can't build a 2d Cartesian coordinate");
        chart
            .configure_mesh()
            .x_labels(10)
            .y_labels(10)
            .disable_mesh()
            .x_label_formatter(&|&v| format!("{:0.1}", v as f32 / self.sample_rate))
            .y_label_formatter(&|v| format!("{:0.1}", v))
            .x_desc("Time [s]")
            .draw()
            .expect("Can't draw axes");
        chart
            .draw_series(LineSeries::new(
                self.audio_data.iter().enumerate().map(|(i, p)| (i, *p)),
                BLUE.filled(),
            ))
            .expect("Can't draw a series");
        chart
            .plotting_area()
            .draw(&Rectangle::new(
                [
                    (0, audio_range.start),
                    (
                        self.spectra
                            .as_ref()
                            .and_then(|spectra| Some(spectra.len()))
                            .unwrap_or_default(),
                        audio_range.end,
                    ),
                ],
                ShapeStyle {
                    color: GREEN.mix(0.3),
                    filled: true,
                    stroke_width: 1,
                },
            ))
            .expect("Can't draw a rectangle");
        if let Some(spectra) = self.spectra.as_ref() {
            let areas = frequency_area.split_evenly((1, 2));
            let power_area = &areas[0];
            let phase_area = &areas[1];

            let powers: Vec<_> = spectra.iter().map(|c| c.norm_sqr()).collect();
            let df = self.sample_rate / powers.len() as f32;
            let mut chart = ChartBuilder::on(&power_area)
                .x_label_area_size(50)
                .y_label_area_size(60)
                .build_cartesian_2d(
                    0..powers.len() / 2,
                    0.0..plotters::data::fitting_range(&powers).end + 0.1,
                )
                .expect("Can't build a 2d Cartesian coordinate");
            chart
                .configure_mesh()
                .x_labels(10)
                .y_labels(10)
                .disable_mesh()
                .x_label_formatter(&|&v| {
                    format!(
                        "{:0.1}",
                        if v < (powers.len() + 1) / 2 {
                            v as f32 * df
                        } else {
                            -((powers.len() - v) as f32) * df
                        }
                    )
                })
                .y_label_formatter(&|v| format!("{:e}", v))
                .x_desc("Frequency [Hz]")
                .y_desc("Power")
                .draw()
                .expect("Can't draw axes");
            chart
                .draw_series(LineSeries::new(
                    powers
                        .iter()
                        .take(powers.len() / 2)
                        .enumerate()
                        .map(|(i, p)| (i, *p)),
                    BLUE.filled(),
                ))
                .expect("Can't draw a series");
            let mut phases: Vec<_> = spectra.iter().map(|c| c.arg()).collect();
            unwrap_phase(&mut phases);
            let df = self.sample_rate / phases.len() as f32;
            let mut chart = ChartBuilder::on(&phase_area)
                .x_label_area_size(50)
                .y_label_area_size(60)
                .build_cartesian_2d(0..phases.len() / 2, plotters::data::fitting_range(&phases))
                .expect("Can't build a 2d Cartesian coordinate");
            chart
                .configure_mesh()
                .x_labels(10)
                .y_labels(10)
                .disable_mesh()
                .x_label_formatter(&|&v| {
                    format!(
                        "{:0.1}",
                        if v < (phases.len() + 1) / 2 {
                            v as f32 * df
                        } else {
                            -((phases.len() - v) as f32) * df
                        }
                    )
                })
                .y_label_formatter(&|v| format!("{:}", v))
                .x_desc("Frequency [Hz]")
                .y_desc("Phase [rad]")
                .draw()
                .expect("Can't draw axes");
            chart
                .draw_series(LineSeries::new(
                    phases
                        .iter()
                        .take(phases.len() / 2)
                        .enumerate()
                        .map(|(i, p)| (i, *p)),
                    BLUE.filled(),
                ))
                .expect("Can't draw a series");
        }
    }
}

fn unwrap_phase(phases: &mut [f32]) {
    let diff: Vec<_> = phases
        .iter()
        .skip(1)
        .scan(phases[0], |state, phase| {
            let d = *phase - *state;
            *state = *phase;
            Some(d)
        })
        .map(|diff| {
            let tmp = ((diff + PI) % (2.0 * PI)) - PI;
            let tmp = if tmp == -PI && diff > 0.0 {
                PI - diff
            } else {
                tmp - diff
            };
            if diff.abs() < PI {
                0.0
            } else {
                tmp
            }
        })
        .scan(0.0, |state, diff| {
            *state += diff;
            Some(*state)
        })
        .collect();
    phases
        .iter_mut()
        .skip(1)
        .zip(diff.iter())
        .for_each(|(phase, diff)| {
            *phase += diff;
        });
}

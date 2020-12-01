use std::f32::consts::PI;

use plotters::{
    chart::ChartBuilder,
    drawing::IntoDrawingArea,
    element::Rectangle,
    style::{colors::WHITE, Color, HSLColor},
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
pub fn run_stft(audio_data: &[f32], size: usize, window_function: WindowFunction) -> Vec<f32> {
    let offset = size / 2;
    let mut fft_planner = FFTplanner::new(false);
    let fft = fft_planner.plan_fft(size);
    let window = window_function.generate(size);

    let mut input: Vec<_> = audio_data
        .windows(size)
        .step_by(offset)
        .flat_map(|x| {
            x.iter()
                .zip(window.iter())
                .map(|(x, w)| Complex::new(x * w, 0.0))
        })
        .collect();
    let mut output = vec![Complex::zero(); input.len()];
    fft.process_multi(&mut input, &mut output);
    output
        .chunks_exact(size)
        .flat_map(|chunk| chunk.iter().flat_map(|c| vec![c.re, c.im]))
        .collect()
}

#[wasm_bindgen]
pub fn draw(canvas: HtmlCanvasElement, spectra: &[f32], size: usize, sample_rate: f32) {
    let root = CanvasBackend::with_canvas_object(canvas)
        .expect("Illegal canvas")
        .into_drawing_area();
    root.fill(&WHITE)
        .expect("Some errors have been occurred in the backend");

    let powers: Vec<Vec<_>> = spectra
        .chunks_exact(2 * size)
        .map(|spectra| {
            spectra
                .chunks_exact(2)
                .take(size / 2 / 5)
                .map(|chunk| chunk[0] * chunk[0] + chunk[1] * chunk[1])
                .collect()
        })
        .collect();
    let max_power = powers.iter().fold(0.0 / 0.0, |max, powers| {
        powers
            .iter()
            .fold(0.0 / 0.0, |max, power| power.max(max))
            .max(max)
    });
    let df = sample_rate / size as f32;
    let size = powers[0].len();

    let mut chart = ChartBuilder::on(&root)
        .x_label_area_size(50)
        .y_label_area_size(80)
        .build_cartesian_2d(0..powers.len(), 0..size)
        .expect("Can't build a 2d Cartesian coordinate");

    chart
        .configure_mesh()
        .x_labels(10)
        .y_labels(10)
        .disable_mesh()
        .y_label_formatter(&|&v| format!("{:0.1}", v as f32 * df))
        .y_desc("Frequency [Hz]")
        .draw()
        .expect("Can't draw axes");

    let drawing_area = chart.plotting_area();
    powers.iter().enumerate().for_each(|(i, powers)| {
        powers.iter().enumerate().for_each(|(j, power)| {
            drawing_area
                .draw(&Rectangle::new(
                    [(i, j), (i + 1, j + 1)],
                    HSLColor(0.0, 1.0, (power.sqrt() / max_power.sqrt()) as f64).filled(),
                ))
                .expect("Can't draw a pixel");
        })
    });
}

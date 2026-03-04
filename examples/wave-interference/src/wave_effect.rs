use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use tachyonfx::{color_from_hsl, default_shader_impl, wave_sin, CellFilter, ColorSpace, Duration, FilterProcessor, Interpolation, Shader};
use tachyonfx::wave::{Modulator, Oscillator, SignalSampler, WaveLayer};

/// A shader that creates wave interference patterns.
#[derive(Debug, Clone)]
pub struct WaveInterference {
    alive: Duration,
    waves: Vec<WaveLayer>,
    total_amplitude: f32,
    area: Option<Rect>,
    cell_filter: Option<FilterProcessor>,
    color_space: ColorSpace,
}

impl WaveInterference {
    /// Creates a new wave interference effect with default settings.
    pub fn new() -> Self {
        let waves = vec![
            WaveLayer::new(
                Oscillator::sin(0.14, 0.0, 0.12)
                    .modulated_by(Modulator::sin(0.113, 0.253, 0.0183).on_amplitude().intensity(0.5)))
                .max(Oscillator::cos(0.2, -0.35, 0.016)
                    .modulated_by(Modulator::sin(-0.00234, 0.1, 0.01333).on_phase().intensity(1.0)))
                .amplitude(0.5),
            WaveLayer::new(Oscillator::sawtooth(0.125, 0.0, -0.04)
                .modulated_by(Modulator::sin(0.1, 0.3, 0.17).intensity(0.2).on_phase()))
                .multiply(Oscillator::sawtooth(0.0, 0.25, 0.3)
                    .modulated_by(Modulator::sin(0.0, 0.13, 0.007).intensity(0.2).on_amplitude()))
                .amplitude(0.4),
            WaveLayer::new(
                Oscillator::sin(-0.12, 0.08, -1.5)
                    .modulated_by(Modulator::sin(0.0, 0.3, 0.7).intensity(0.2).on_phase()))
                .multiply(Oscillator::sin(-0.01, 0.07813, 0.8253)
                    .modulated_by(Modulator::cos(-0.00234, 0.1, 0.1333).on_phase()))
                .amplitude(1.2),
            WaveLayer::new(
                Oscillator::sawtooth(1.4, 0.2, 0.4)
                    .modulated_by(Modulator::sin(0.5, 0.0, 10.2).intensity(0.4).on_phase()))
                .multiply(Oscillator::sin(0.0, 0.2, 0.8))
                .amplitude(0.3),
            WaveLayer::new(Oscillator::sin(0.4, 0.025, 0.302)
                    .modulated_by(Modulator::triangle(0.0, 0.3, 0.7).on_phase()))
                .max(Oscillator::cos(0.0132, -0.35, -0.06)
                    .modulated_by(Modulator::sawtooth(0.4, 0.03, 0.1).on_phase()))
                .amplitude(1.2),
            WaveLayer::new(Oscillator::sin(-0.1, -0.17325, 0.711)
                    .modulated_by(Modulator::cos(-0.05672, 0.00961, 0.4333).on_amplitude()))
                .max(Oscillator::cos(-0.012, -0.35, -0.6)
                    .modulated_by(Modulator::cos(-0.00234, 0.1, 0.1333).on_amplitude()))
                .amplitude(1.2)
                .abs(),
        ];

        let total_amplitude = waves.iter().map(|w| w.amplitude_value()).sum::<f32>();

        Self {
            alive: Duration::from_millis(0),
            waves,
            total_amplitude,
            area: None,
            cell_filter: None,
            color_space: ColorSpace::Hsl,
        }
    }
}

fn calc_wave_amplitude(
    elapsed: f32,
    pos: (f32, f32),
    waves: &[WaveLayer],
    total_amplitude: f32,
) -> f32 {
    waves
        .iter()
        .map(|w| w.sample(pos.0, pos.1, elapsed))
        .sum::<f32>()
        / total_amplitude
}

impl Shader for WaveInterference {
    default_shader_impl!(area, clone, color_space);

    fn name(&self) -> &'static str {
        "wave_interference"
    }

    fn process(&mut self, duration: Duration, buf: &mut Buffer, area: Rect) -> Option<Duration> {
        self.alive += duration;
        let elapsed = self.alive.as_secs_f32();
        let waves = self.waves.clone();
        let total_amplitude = self.total_amplitude;

        let elapsed_cos = elapsed.cos();

        // let l_wave = WaveLayer::new(
        //     // Oscillator::sin(0.4, 0.0, 0.12 * elapsed_cos)
        //     Oscillator::sin(0.4, 0.0, 0.12)
        //         .modulated_by(Modulator::sin(0.1, 0.25, 0.08 * elapsed_cos).on_phase().intensity(1.5)))
        //     .max(Oscillator::cos(0.2, -0.35, 0.16 * elapsed_cos))
        //     .power(2)
        //     .amplitude(0.25)
        //     .abs();

        let hue_wave = WaveLayer::new(
            Oscillator::sin(0.14, 0.25, 1.2)
                .modulated_by(Modulator::sin(0.0, -0.25, 0.02 * elapsed_cos).on_phase().intensity(2.5)))
            .multiply(Oscillator::cos(-0.2, -0.35, 0.01 * elapsed_cos).phase(elapsed))
            // .power(2)
            .amplitude(0.6);

        self.cell_iter(buf, area).for_each_cell(|pos, cell| {
            let pos = (pos.x as f32, pos.y as f32);
            let normalized = calc_wave_amplitude(elapsed, pos, &waves, total_amplitude)
                .clamp(-1.0, 1.0);

            let a = Interpolation::BackOut.alpha(normalized.abs()) * normalized.signum();
            let hue_shift = hue_wave.sample(pos.0, pos.1, elapsed) * 20.0 + (elapsed_cos * 5.0);

            let hue = (
                (normalized * normalized) * 270.0
                    + hue_shift
                    - (0.4 * pos.0 * elapsed_cos * a)
                    - (1.0 * pos.1 * elapsed_cos * a))
                .rem_euclid(360.0);
            let a = a.clamp(0.0, 1.0);
            let lightness = -50.0 + (a * a * a) * 90.0;
            let saturation = -10.0 + (a * a * 120.0) - ((120.0 - lightness) * 0.5);

            let saturation = saturation.clamp(0.0, 100.0);
            let lightness = lightness.clamp(0.0, 100.0);

            cell.set_bg(color_from_hsl(
                (hue + elapsed * 10.0).rem_euclid(360.0),
                saturation,
                lightness,
            ));
        });

        None
    }

    fn done(&self) -> bool {
        false
    }

    fn filter(&mut self, strategy: CellFilter) {
        self.cell_filter = Some(FilterProcessor::from(strategy));
    }

    fn cell_filter(&self) -> Option<&CellFilter> {
        self.cell_filter.as_ref().map(|f| f.filter_ref())
    }

    fn filter_processor(&self) -> Option<&FilterProcessor> {
        self.cell_filter.as_ref()
    }

    fn filter_processor_mut(&mut self) -> Option<&mut FilterProcessor> {
        self.cell_filter.as_mut()
    }

    fn reset(&mut self) {
        self.alive = Duration::from_secs(0);
    }
}

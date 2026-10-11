mod graphic_eq;

use anyhow::Result;
use fundsp::prelude64::*;

fn main() -> Result<()> {
    // --- CONFIGURATION ---
    let input_path = "phoneme-data/james/james.wav";
    let output_path = "funky.wav";

    // Low Shelf: Frequency (Hz), Gain (dB)
    let low_freq = 300.0;
    let low_gain = 0.001;
    let low_q = 1.0;

    // Peaking: Frequency (Hz), Gain (dB), Q (Quality factor)
    let mid_freq = 1950.0;
    let mid_gain = 1000.0;
    let mid_q = 0.5;

    // High Shelf: Frequency (Hz), Gain (dB)
    let high_freq = 3300.0;
    let high_gain = 0.001;
    let high_q = 1.0;
    // ---------------------

    println!("Reading: {}", input_path);
    let input_audio = Wave::load(&input_path)?;

    println!("Applying EQ...");
    // Create the fundsp graph using the DSL
    // We chain the filters using the >> operator
    let mut eq_chain = 
        lowshelf_hz(low_freq, low_q, low_gain) >> 
        bell_hz(mid_freq, mid_q, mid_gain) >>
        highshelf_hz(high_freq, high_q, high_gain);

    let mut output_audio = input_audio.filter(
        (input_audio.length() as f64 / input_audio.sample_rate()) as f64,
        &mut eq_chain
    );

    output_audio.normalize();
    output_audio.save_wav16(output_path)
        .expect("Failed to save the output WAV file.");

    println!("Done!");
    Ok(())
}

mod test {
use fundsp::prelude::*;

use insta_fun::{assert_audio_unit_meta_data_snapshot, assert_audio_unit_snapshot};
use insta_fun::config::{
    SvgChartConfigBuilder, SvgPreserveAspectRatio, SvgPreserveAspectRatioAlignment,
    SvgPreserveAspectRatioBuilder, SvgPreserveAspectRatioKwd, WavOutput,
};
use insta_fun::prelude::*;

    #[test]
    fn test_custom_input() {
        let chart_cfg = SvgChartConfigBuilder::default()
            .chart_title("Custom Sine Chart")
            .show_grid(true)
            .build()
            .unwrap();
        let config = SnapshotConfigBuilder::default()
            .num_samples(2048)
            .output_mode(chart_cfg)
            .build()
            .unwrap();
        let input = (0..2048).map(|i| (i as f32 / 50.0).sin()).collect();
    
        let data = snapshot_audio_unit_with_input_and_options(
            lowpass_hz(5.0, 0.7),
            InputSource::VecByChannel(vec![input]),
            config,
        );

        std::fs::write("output.svg", data).expect("Brugh");
    }
}

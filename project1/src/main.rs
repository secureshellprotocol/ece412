mod graphic_eq;
mod pitchshift_sampling;
mod compressor;

use anyhow::Result;
use fundsp::prelude64::*;
use graphic_eq::GraphicEqBlock;
use pitchshift_sampling::SamplerShiftBlock;

fn main() -> Result<()> {
    // --- CONFIGURATION ---
    let input_path = "basement_impulse.wav";
    let name_without_ext = input_path
        .rsplit_once('.')
        .map(|(prefix, _)| prefix)
        .unwrap_or(input_path);
    let g_eq = GraphicEqBlock::default();

    println!("Reading: {}", input_path);
    let mut input_audio = Wave::load(&input_path)?;
    // only want one channel
    if input_audio.channels() > 1 {
        for i in 1..input_audio.channels() {
            input_audio.remove_channel(i);
        }
    }

    //if let Ok(output) = g_eq.work(input_audio) {
    //if let Ok(output) = SamplerShiftBlock::shift_by_factor(5.0, input_audio, true) {
    //    let output_path = format!("{}_mod.wav", name_without_ext);
    //    println!("Wrote output to {}", output_path);
    //    output.save_wav16(output_path)
    //        .expect("Failed to save the output WAV file.");
    //}



    println!("Press any key to continue...");
    // Read the line from standard input
    std::io::stdin()
        .read_line(&mut String::new())
        .expect("Failed to read line");

    println!("Closing...");
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

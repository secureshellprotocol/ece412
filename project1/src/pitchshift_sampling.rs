use anyhow::Result;
use fundsp::prelude::*;

pub struct SamplerShiftBlock {}
impl SamplerShiftBlock {
    pub fn shift_by_factor(factor: f64, input_audio: Wave, antialias: bool) -> Result<Wave> {
        let original_sampling_rate = input_audio.sample_rate();

        let new_sampling_rate = original_sampling_rate * factor;

        let mut resampled_audio: Wave = input_audio.clone();
        resampled_audio.set_sample_rate(new_sampling_rate);

        if antialias {
            resampled_audio = resampled_audio.filter(
                (resampled_audio.length() as f64 / new_sampling_rate) as f64,
                &mut lowshelf_hz(original_sampling_rate, 1.0, 1.0)
            );
        }
        return Ok(resampled_audio);
    }
}

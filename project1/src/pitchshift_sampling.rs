use anyhow::Result;
use fundsp::prelude::*;
use serde::{Serialize, Deserialize};

const SAMPLER_CONFIG_PATH: &str = "./config/sampler_config.yaml";

#[derive(Serialize, Deserialize, Debug)]
pub struct SamplerShiftBlock {
    enabled: bool,

    /// (0.0, 1.0) will stretch the signal in time and drop the pitch
    /// (1.0, inf) will shrink the signal in time and raise the pitch
    /// 1.0 is unity
    shift_factor: f64,
    /// We filter against our original sampling frequuency to remove any 
    /// aliased components which may have gotten into our signal
    filter: bool
}
impl SamplerShiftBlock {
    pub fn new() -> SamplerShiftBlock {
        match std::fs::exists(SAMPLER_CONFIG_PATH) {
            Ok(true) => {
                yaml_serde::from_str(
                    &std::fs::read_to_string(SAMPLER_CONFIG_PATH.to_owned()).unwrap()
                ).unwrap_or(SamplerShiftBlock::default())
            }
            Ok(false) => {
                SamplerShiftBlock::default()
            }
            Err(_) => {
                SamplerShiftBlock::default()
            }
        }
    }

    pub fn work(&self, input_audio: Wave) -> Result<Wave> {
        print!("Applying pitch shifting / time stretching...");
        let mut input_audio = input_audio.clone();
        let original_sampling_rate = input_audio.sample_rate();

        let new_sampling_rate = original_sampling_rate * self.shift_factor;

        //anti image: remove stuff that would seep into our band during downsample
        if self.filter && self.shift_factor < 1.0 {
            input_audio = input_audio.filter(
                (input_audio.length() as f64 / original_sampling_rate) as f64,
                &mut lowshelf_hz(new_sampling_rate / 2.0, 1.0, 1.0)
            );
        }

        // the step -- raise or lower the sampling rate based on the shift factor.
        // note: This internally i think is just changing the shift freq. used by 
        // the wav format. it didnt seem like this library exposed a lower level 
        // for doing a matlab-esque zero over/undersample..
        let mut resampled_audio: Wave = input_audio.clone();
        resampled_audio.set_sample_rate(new_sampling_rate);

        //anti alias: remove stuff that would seep into our band duuring upsample
        if self.filter && self.shift_factor > 1.0 {
            resampled_audio = resampled_audio.filter(
                (resampled_audio.length() as f64 / new_sampling_rate) as f64,
                &mut lowshelf_hz(original_sampling_rate / 2.0, 1.0, 1.0)
            );
        }
        resampled_audio.normalize();
        println!("done!");
        return Ok(resampled_audio);
    }
}
impl Default for SamplerShiftBlock {
    fn default() -> Self {
        SamplerShiftBlock {
            enabled: true,
            shift_factor: 1.0,
            filter: true,
        }
    }
}

mod test {
    #[allow(unused)]
    use super::*;
    #[test]
    fn make_sampl_settings() -> Result<(), Box<dyn std::error::Error>> {
        let yaml_string = yaml_serde::to_string(&SamplerShiftBlock::default())?;
        std::fs::write(SAMPLER_CONFIG_PATH, yaml_string)?;
        Ok(())
    }
}

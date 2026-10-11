use anyhow::Result;
use fundsp::prelude::*;
use serde::{Serialize, Deserialize};

const COMP_CONFIG_PATH: &str = "./config/compressor_config.yaml";

#[derive(Serialize, Deserialize, Clone)]
pub struct CompressorBlock {
    /// Maximum dB our signal can exceed
    ///     We adapt and try to bring this down
    threshold: f32,
    /// How aggressive to be when attenuating the signal 
    ratio: f32,
    attack_ms: f32,
    release_ms: f32,
}

impl CompressorBlock {
    pub fn new() -> Self {
        match std::fs::exists(COMP_CONFIG_PATH) {
            Ok(true) => {
                yaml_serde::from_str(
                    &std::fs::read_to_string(COMP_CONFIG_PATH.to_owned()).unwrap()
                ).unwrap_or(Self::default())
            }
            Ok(false) => {
                Self::default()
            }
            Err(_) => {
                Self::default()
            }
        }
    }
}
impl CompressorBlock {
    pub fn work(&self, input: Wave) -> Result<Wave> {
        // Max abs. magnitude to exceed
        let sample_rate = input.sample_rate() as f32;
        let mut envelope = 0.0;
        let threshold_db = 20.0 * self.threshold.log10();

        let attack_coeff = 1.0 - (-1.0 / (self.attack_ms * 0.001 * sample_rate)).exp();
        let release_coeff = 1.0 - (-1.0 / (self.release_ms * 0.001 * sample_rate)).exp();

        println!("attack {} release {}", attack_coeff, release_coeff);

        print!("compressing...");
        print!("{} channels...",  input.channels());
        let mut input_clone = input.clone();
        let input_channel: &mut [f32] = input_clone.channel_mut(0);

        let mut i = 0;
        while i < input_channel.len() {

            let sample = input_channel[i];
            print!("was {}, ", sample);

            let abs_input = sample.abs();
            
            // Do we need to start attenuating
            let coeff = if abs_input > envelope { 
                attack_coeff 
            } else { 
                release_coeff 
            };
            envelope = (1.0 - coeff) * envelope + coeff * abs_input;

            // gain reduction
            // we set some kind of target threshold, and the ratio basically defines 
            //  our tolerance between that threshold and our final gain
            // making the ratio more aggressive means we are dragged closer to the 
            //  threshold
            // This is all determined by the envelope, which is basically tracing 
            //  top of the signal, and determining whether we need to start backing 
            //  off our envelope (releasing signal mag) or aggressing our envelope (
            //  dropping signal mag)
            let mut gain_reduction_db = 0.0;
            let env_db = 20.0 * envelope.max(1e-5).log10();
            if env_db > threshold_db { 
                gain_reduction_db = (env_db - threshold_db) * (1.0 / self.ratio - 1.0);
            }
            println!("DEBUG: Sample: {:.4}, Env_dB: {:.2}, Threshold: {:.2}, Gain_Red: {:.2}", 
                      sample, env_db, threshold_db, gain_reduction_db);

            let gain_multiplier = 10.0f32.powf(gain_reduction_db / 20.0);

            input_channel[i] = sample * gain_multiplier;
            println!("now {}", input_channel[i]);
            i = i + 1;
        }

        println!("done!");
        return Ok(Wave::from_samples(sample_rate as f64, input_channel));
    }
}
impl Default for CompressorBlock {
    fn default() -> CompressorBlock {
        CompressorBlock {
            threshold: 0.001,
            ratio: 4.0,
            attack_ms: 10.0,
            release_ms: 100.0,
        }
    }
}
mod test {
    #[allow(unused)]
    use super::*;
    #[test]
    fn make_compressor_settings() -> Result<(), Box<dyn std::error::Error>> {
        let yaml_string = yaml_serde::to_string(&CompressorBlock::default())?;
        std::fs::write(COMP_CONFIG_PATH, yaml_string)?;
        Ok(())
    }
}

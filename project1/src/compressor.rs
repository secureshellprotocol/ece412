use anyhow::Result;
use fundsp::prelude::*;

#[derive(Clone)]
pub struct CompressorBlock {
    threshold_db: f32,
    ratio: f32,
    attack_coeff: f32,
    release_coeff: f32,
    envelope: f32,
    sample_rate: f32,
}

impl CompressorBlock {
    pub fn new(
        threshold: f32,
        ratio: f32,
        attack_ms: f32,
        release_ms: f32,
        sample_rate: f32,
    ) -> Self {
        let attack_coeff = 1.0 - (-1.0 / (attack_ms * 0.001 * sample_rate)).exp();
        let release_coeff = 1.0 - (-1.0 / (release_ms * 0.001 * sample_rate)).exp();

        println!("attack {} release {}", attack_coeff, release_coeff);

        Self {
            threshold_db: 10.0 * threshold.log10(),
            ratio,
            attack_coeff,
            release_coeff,
            envelope: 0.0,
            sample_rate,
        }
    }
}

impl CompressorBlock {
    pub fn work(&mut self, input: Wave) -> Result<Wave> {
        print!("compressing...");
        print!("{} channels...",  input.channels());
        let mut input_clone = input.clone();
        let input_channel: &mut [f32] = input_clone.channel_mut(0);

        let mut i = 0;
        while i < input_channel.len() {

            let sample = input_channel[i];
            print!("was {}, ", sample);

            let abs_input = sample.abs();
            
            // envelope detection
            let coeff = if abs_input > self.envelope { 
                self.attack_coeff 
            } else { 
                self.release_coeff 
            };
            self.envelope = (1.0 - coeff) * self.envelope + coeff * abs_input;
            let env_db = 20.0 * self.envelope.max(1e-5).log10();

            // gain reduction
            let mut gain_reduction_db = 0.0;
            if env_db > self.threshold_db {
                gain_reduction_db = (env_db - self.threshold_db) * (1.0 / self.ratio - 1.0);
            }
            // ADD THIS DEBUG LINE:
            println!("DEBUG: Sample: {:.4}, Env_dB: {:.2}, Threshold: {:.2}, Gain_Red: {:.2}", 
                      sample, env_db, self.threshold_db, gain_reduction_db);

            let gain_multiplier = 10.0f32.powf(gain_reduction_db / 20.0);

            input_channel[i] = sample * gain_multiplier;
            println!("now {}", input_channel[i]);
            i = i + 1;
        }

        println!("done!");
        return Ok(Wave::from_samples(self.sample_rate as f64, input_channel));
    }
}


use anyhow::Result;
use fundsp::prelude64::*;
use serde::{Serialize, Deserialize};
use rand::thread_rng;
use rand_distr::{Distribution, Exp};

const REVERB_CONFIG_PATH: &str = "./config/reverb_config.yaml";

// length in samples = length in time / sampling frequency
// 0.5 seconds * 48000 samples/second = 24000 samples
// in post: never meet your heroes... they implement the conversion in powers of
// 10, so we can only use factors of 10e{x} for array sizes. see wtf.png
const REVERB_LENGTH_SAMPLES: usize = 10000;

#[derive(Serialize, Deserialize, Debug)]
pub struct ReverbBlock {
    enabled: bool,

    decay_factor: f32,
}
impl ReverbBlock {
    pub fn new() -> ReverbBlock {
        match std::fs::exists(REVERB_CONFIG_PATH) {
            Ok(true) => {
                yaml_serde::from_str(
                    &std::fs::read_to_string(REVERB_CONFIG_PATH.to_owned()).unwrap()
                ).unwrap_or(ReverbBlock::default())
            }
            Ok(false) => {
                ReverbBlock::default()
            }
            Err(_) => {
                ReverbBlock::default()
            }
        }
    }

    pub fn work(&self, input_audio: Wave) -> Result<Wave> {
        if !self.enabled {
            println!("Reverb disabled, skipping...");
            return Ok(input_audio);
        }

        print!("Applying reverb...");
    
        let exp = Exp::new(self.decay_factor).unwrap();
        let mut rng = thread_rng();
        let coefficients: Vec<f32> = (0..REVERB_LENGTH_SAMPLES)
            .map(|_| exp.sample(&mut rng))
            .collect();

        let coefficients: [f32; REVERB_LENGTH_SAMPLES] = 
            coefficients[..].try_into().unwrap();

        let mut reverb_filter = fir(Frame::<f32, _>::from(coefficients));

        let mut output_audio = input_audio.filter(
            (input_audio.length() as f64 / input_audio.sample_rate()) as f64,
            &mut reverb_filter
        );
    
        output_audio.normalize();

        println!("done!");
        return Ok(output_audio);
    }
}
impl Default for ReverbBlock {
    fn default() -> Self {
        ReverbBlock {
            enabled: true,

            decay_factor: 2.0,
        }
    }
}

mod test {
    #[allow(unused)]
    use super::*;
    #[test]
    fn make_reverb_settings() -> Result<(), Box<dyn std::error::Error>> {
        let yaml_string = yaml_serde::to_string(&ReverbBlock::default())?;
        std::fs::write(REVERB_CONFIG_PATH, yaml_string)?;
        Ok(())
    }
}

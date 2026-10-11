use fundsp::prelude::*;

#[derive(Clone)] // Required by AudioNode trait
pub struct ManualCompressor {
    threshold_db: f32,
    ratio: f32,
    attack_coeff: f32,
    release_coeff: f32,
    envelope: f32,
}

impl ManualCompressor {
    /// Constructor
    /// threshold: dB, ratio: n:1, attack_ms: ms, release_ms: ms, sample_rate: Hz
    pub fn new(
        threshold_db: f32,
        ratio: f32,
        attack_ms: f32,
        release_ms: f32,
        sample_rate: f32,
    ) -> Self {
        let attack_coeff = 1.0 - (-1.0 / (attack_ms * 0.001 * sample_rate)).exp();
        let release_coeff = 1.0 - (-1.0 / (release_ms * 0.001 * sample_rate)).exp();

        Self {
            threshold_db,
            ratio,
            attack_coeff,
            release_coeff,
            envelope: 0.0,
        }
    }
}

// 1. Implement AudioNode without generics (unless you want multi-channel)
impl AudioNode for ManualCompressor {
    // 2. Define associated types for Mono (Inputs = (), Outputs = ())
    type Inputs = U1;
    type Outputs = U1;

    // 3. Required constant ID
    const ID: u64 = 42;

    // 4. The correct tick signature for fundsp 0.23.0
    fn tick(&mut self, input: &Frame<f32, Self::Inputs>) -> Frame<f32, Self::Outputs> {
        // Access the mono sample from the input Frame
        let sample = input[0];

        // --- DSP Logic ---
        let abs_input = sample.abs();
        
        // Envelope Follower (Attack/Release)
        let coeff = if abs_input > self.envelope {
            self.attack_coeff
        } else {
            self.release_coeff
        };
        self.envelope = (1.0 - coeff) * self.envelope + coeff * abs_input;

        // Convert envelope to dB
        // We use a floor to prevent log10(0)
        let env_db = 20.0 * self.envelope.max(1e-5).log10();
        
        let mut gain_reduction_db = 0.0;
        if env_db > self.threshold_db {
            gain_reduction_db = (env_db - self.threshold_db) * (1.0 / self.ratio - 1.0);
        }

        let gain_multiplier = 10.0f32.powf(gain_reduction_db / 20.0);

        // Return the processed sample as a mono Frame
        [sample * gain_multiplier].into()
    }
}


#[cfg(test)]
mod test {
    use fundsp::prelude::*;
    use super::*;

    #[test]
    fn awesome() {
        let sample_rate = 44100.0;
        
        // Now you can use it exactly like a native fundsp node!
        let compressor = ManualCompressor::new(-20.0, 4.0, 10.0, 100.0, sample_rate);
        
        // This works because the compressor is an "Effect" node in the chain
        let mut graph: Chain<Sine<f32>, ManualCompressor> = sin(440.0) >> compressor;

        for _ in 0..10 {
            println!("Sample: {}", graph.tick());
        }
    }
}

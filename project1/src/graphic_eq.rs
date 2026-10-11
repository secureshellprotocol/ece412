use anyhow::Result;
use fundsp::prelude64::*;

pub struct GraphicEqBlock {
    /// Bass filter params
    low_freq: f32,  // Hz     
    low_gain: f32,  // Linear 
    low_q: f32,     // Q      

    /// Mid filter params
    mid_freq: f32,  // Hz     
    mid_gain: f32,  // Linear
    mid_q: f32,     // Q

    /// High filter params
    high_freq: f32, // Hz     
    high_gain: f32, // Linear
    high_q: f32,    // Q
}
impl GraphicEqBlock {
    pub fn work(&self, input_audio: Wave) -> Result<Wave> {
        println!("Applying EQ...");
        // Create the fundsp graph using the DSL
        // We chain the filters using the >> operator
        let mut eq_chain = lowshelf_hz(self.low_freq, self.low_q, self.low_gain) >> 
            bell_hz(self.mid_freq, self.mid_q, self.mid_gain) >>
            highshelf_hz(self.high_freq, self.high_q, self.high_gain);
    
        let mut output_audio = input_audio.filter(
            (input_audio.length() as f64 / input_audio.sample_rate()) as f64,
            &mut eq_chain
        );
    
        output_audio.normalize();
    
        println!("Eq done!");
        return Ok(output_audio);
    }
}
impl Default for GraphicEqBlock {
    fn default() -> Self {
        GraphicEqBlock {
            low_freq: 300.0,
            low_gain: 0.01,
            low_q: 1.0,

            mid_freq: 1950.0,
            mid_gain: 1000.0,
            mid_q: 0.5,

            high_freq: 3300.0,
            high_gain: 0.01,
            high_q: 1.0,
        }
    }
}


use anyhow::Result;
use fundsp::prelude64::*;

pub struct GraphicEqBlock {
    /// Low pass parameters
    let low_freq = 300.0;  // Hz     
    let low_gain = 0.001;  // Linear 
    let low_q = 1.0;       // Q      

    /// 
    let mid_freq = 1950.0; // Hz     
    let mid_gain = 1000.0; // Linear
    let mid_q = 0.5;       // Q

    let high_freq = 3300.0; // Hz     
    let high_gain = 0.001;  // Linear
    let high_q = 1.0;       // Q


}

impl GraphicEqBlock {
    fn work(input_audio: Wave) -> Result<Wave> {
        // --- CONFIGURATION ---
        let input_path = "phoneme-data/james/james.wav";
        let output_path = "funky.wav";
    
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
        return output_wave;
    }
}



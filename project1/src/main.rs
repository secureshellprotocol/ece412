#[derive(Clone)]
pub struct Biquad {
    // Coefficients
    b0: f32, b1: f32, b2: f32,
    a1: f32, a2: f32,
    // State (Direct Form I)
    x1: f32, x2: f32,
    y1: f32, y2: f32,
}

impl Biquad {
    fn new() -> Self {
        Self {
            b0: 1.0, b1: 0.0, b2: 0.0, a1: 0.0, a2: 0.0,
            x1: 0.0, x2: 0.0, y1: 0.0, y2: 0.0,
        }
    }

    #[inline]
    pub fn process(&mut self, x: f32) -> f32 {
        let y = (self.b0 * x) + (self.b1 * self.x1) + (self.b2 * self.x2)
                - (self.a1 * self.y1) - (self.a2 * self.y2);

        self.x2 = self.x1;
        self.x1 = x;
        self.y2 = self.y1;
        self.y1 = y;
        y
    }

    /// The core calculation engine. 
    /// Uses f64 for all intermediate math and applies stability clamps.
    fn update_coeffs(&mut self, b0: f64, b1: f64, b2: f64, a0: f64, a1: f64, a2: f64) {
        // 1. Normalize all coefficients by a0
        let nb0 = b0 / a0;
        let nb1 = b1 / a0;
        let nb2 = b2 / a0;
        let na1 = a1 / a0;
        let na2 = a2 / a0;

        // 2. STABILITY GUARD:
        // In a Biquad filter, if |a1| or |a2| >= 1.0, the filter explodes (becomes an oscillator).
        // We clamp them to 0.999 to ensure mathematical stability.
        let na1_clamped = na1.clamp(-0.999, 0.999);
        let na2_clamped = na2.clamp(-0.999, 0.999);

        self.b0 = nb0 as f32;
        self.b1 = nb1 as f32;
        self.b2 = nb2 as f32;
        self.a1 = na1_clamped as f32;
        self.a2 = na2_clamped as f32;
    }

    pub fn set_peaking(&mut self, sr: f32, freq: f32, q: f32, gain_db: f32) {
        let (sr, f, q, db) = (sr as f64, freq as f64, q as f64, gain_db as f64);
        let a = 10.0f64.powf(db / 40.0);
        let omega = 2.0 * std::f32::consts::PI as f64 * f / sr;
        let alpha = omega.sin() / (2.0 * q);
        let cos_w = omega.cos();

        let b0 = 1.0 + alpha * a;
        let b1 = -2.0 * cos_w;
        let b2 = 1.0 - alpha * a;
        let a0 = 1.0 + alpha / a;
        let a1 = -2.0 * cos_w;
        let a2 = 1.0 - alpha / a;

        self.update_coeffs(b0, b1, b2, a0, a1, a2);
    }

    pub fn set_low_shelf(&mut self, sr: f32, freq: f32, q: f32, gain_db: f32) {
        let (sr, f, q, db) = (sr as f64, freq as f64, q as f64, gain_db as f64);
        let a = 10.0f64.powf(db / 40.0);
        let omega = 2.0 * std::f32::consts::PI as f64 * f / sr;
        let sin_w = omega.sin();
        let cos_w = omega.cos();
        let alpha = (sin_w / 2.0) * ((a + 1.0/a).sqrt() * (1.0/q - 1.0).sqrt() + 2.0).sqrt();

        let b0 = a * ((a + 1.0) - (a - 1.0) * cos_w + 2.0 * a.sqrt() * alpha);
        let b1 = 2.0 * a * ((a - 1.0) - (a + 1.0) * cos_w + 2.0 * a.sqrt() * alpha);
        let b2 = a * ((a + 1.0) - (a - 1.0) * cos_w - 2.0 * a.sqrt() * alpha);
        let a0 = (a + 1.0) + (a - 1.0) * cos_w + 2.0 * a.sqrt() * alpha;
        let a1 = -2.0 * ((a - 1.0) + (a + 1.0) * cos_w + 2.0 * a.sqrt() * alpha);
        let a2 = (a + 1.0) + (a - 1.0) * cos_w - 2.0 * a.sqrt() * alpha;

        self.update_coeffs(b0, b1, b2, a0, a1, a2);
    }

    pub fn set_high_shelf(&mut self, sr: f32, freq: f32, q: f32, gain_db: f32) {
        let (sr, f, q, db) = (sr as f64, freq as f64, q as f64, gain_db as f64);
        let a = 10.0f64.powf(db / 40.0);
        let omega = 2.0 * std::f32::consts::PI as f64 * f / sr;
        let sin_w = omega.sin();
        let cos_w = omega.cos();
        let alpha = (sin_w / 2.0) * ((a + 1.0/a).sqrt() * (1.0/q - 1.0).sqrt() + 2.0).sqrt();

        let b0 = a * ((a + 1.0) + (a - 1.0) * cos_w + 2.0 * a.sqrt() * alpha);
        let b1 = -2.0 * a * ((a - 1.0) + (a + 1.0) * cos_w + 2.0 * a.sqrt() * alpha);
        let b2 = a * ((a + 1.0) + (a - 1.0) * cos_w - 2.0 * a.sqrt() * alpha);
        let a0 = (a + 1.0) - (a - 1.0) * cos_w + 2.0 * a.sqrt() * alpha;
        let a1 = 2.0 * ((a - 1.0) - (a + 1.0) * cos_w + 2.0 * a.sqrt() * alpha);
        let a2 = (a + 1.0) - (a - 1.0) * cos_w - 2.0 * a.sqrt() * alpha;

        self.update_coeffs(b0, b1, b2, a0, a1, a2);
    }
}

// --- [GraphicEqualizer remains the same] ---
pub struct GraphicEqualizer {
    bass_filters: Vec<Biquad>,
    mid_filters: Vec<Biquad>,
    treble_filters: Vec<Biquad>,
    sample_rate: f32,
    channels: usize,
}

impl GraphicEqualizer {
    pub fn new(sample_rate: f32, channels: usize) -> Self {
        let mut bass = Vec::with_capacity(channels);
        let mut mid = Vec::with_capacity(channels);
        let mut treble = Vec::with_capacity(channels);
        for _ in 0..channels {
            bass.push(Biquad::new());
            mid.push(Biquad::new());
            treble.push(Biquad::new());
        }
        Self { bass_filters: bass, mid_filters: mid, treble_filters: treble, sample_rate, channels }
    }

    pub fn set_bass(&mut self, freq: f32, q: f32, gain_db: f32) {
        for f in &mut self.bass_filters { f.set_low_shelf(self.sample_rate, freq, q, gain_db); }
    }

    pub fn set_mid(&mut self, freq: f32, q: f32, gain_db: f32) {
        for f in &mut self.mid_filters { f.set_peaking(self.sample_rate, freq, q, gain_db); }
    }

    pub fn set_treble(&mut self, freq: f32, q: f32, gain_db: f32) {
        for f in &mut self.treble_filters { f.set_high_shelf(self.sample_rate, freq, q, gain_db); }
    }

    pub fn process(&mut self, audio: &mut [f32]) {
        for chunk in audio.chunks_exact_mut(self.channels) {
            for (i, sample) in chunk.iter_mut().enumerate() {
                let mut s = *sample;
                s = self.bass_filters[i].process(s);
                s = self.mid_filters[i].process(s);
                s = self.treble_filters[i].process(s);
                
                // Safety clamp: If math does go wild, prevent it from becoming NaN/Inf
                // which would ruin the rest of the buffer.
                if s.is_finite() {
                    *sample = s;
                } else {
                    *sample = 0.0; 
                }
            }
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let input_path = "phoneme-data/james/james.wav";
    let output_path = "funky.wav";

    let mut reader = hound::WavReader::open(input_path)?;
    let spec = reader.spec();
    let sample_rate = spec.sample_rate as f32;
    let channels = spec.channels as usize;

    println!("File Format: {:?}, Channels: {}, Sample Rate: {}Hz", spec.sample_format, channels, sample_rate);

    // --- STEP 1: Robust Format-Agnostic Reading ---
    let mut audio_data: Vec<f32> = match spec.sample_format {
        hound::SampleFormat::Int => {
            // Handle 16-bit or 24-bit (as i32)
            if spec.bits_per_sample == 16 {
                reader.samples::<i16>()
                    .map(|s| s.unwrap() as f32 / 32768.0)
                    .collect()
            } else {
                // 24-bit is usually stored in i32
                reader.samples::<i32>()
                    .map(|s| s.unwrap() as f32 / 2147483648.0)
                    .collect()
            }
        }
        hound::SampleFormat::Float => {
            // Handle 32-bit Float files directly
            reader.samples::<f32>()
                .map(|s| s.unwrap())
                .collect()
        }
    };

    if audio_data.is_empty() {
        panic!("No audio data found in file!");
    }

    // --- STEP 2: Sanity Check (Input) ---
    let max_in = audio_data.iter().map(|s| s.abs()).fold(0.0, f32::max);
    println!("Input Audio Max Amplitude: {:.6}", max_in);
    if max_in < 0.00001 {
        println!("WARNING: Input audio is effectively silent. Check your source file.");
    }

    // --- STEP 3: Apply EQ ---
    let mut eq = GraphicEqualizer::new(sample_rate, channels);
    
    // Adjust these values to see if anything changes
    eq.set_bass(100.0, 0.7, 6.0); 
    eq.set_mid(1000.0, 1.0, -3.0);
    eq.set_treble(5000.0, 0.7, 4.0);

    println!("Applying EQ...");
    eq.process(&mut audio_data);

    // --- STEP 4: Sanity Check (Output) ---
    let max_out = audio_data.iter().map(|s| s.abs()).fold(0.0, f32::max);
    println!("Output Audio Max Amplitude: {:.6}", max_out);

    // Check for NaN (Not a Number) - if this happens, the math "exploded"
    if max_out.is_nan() {
        panic!("ERROR: EQ produced NaN (Not a Number). The math is unstable.");
    }

    // --- STEP 5: Write Output ---
    // Note: We write back as i16 for maximum compatibility with players
    let mut writer = hound::WavWriter::create(output_path, hound::WavSpec {
        channels: spec.channels,
        sample_rate: spec.sample_rate,
        sample_format: hound::SampleFormat::Int,
        bits_per_sample: 16,
    })?;

    for sample in audio_data {
        let clamped = sample.clamp(-1.0, 1.0);
        writer.write_sample((clamped * 32767.0) as i16)?;
    }
    writer.finalize()?;

    println!("Successfully saved to {}", output_path);
    Ok(())
}

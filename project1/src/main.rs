#[derive(Clone)]
pub struct Biquad {
    // Coefficients
    b0: f32, b1: f32, b2: f32,
    a1: f32, a2: f32,
    // State variables (Only 2 needed for Direct Form II Transposed)
    z1: f32, z2: f32,
}

impl Biquad {
    fn new() -> Self {
        Self {
            b0: 1.0, b1: 0.0, b2: 0.0, a1: 0.0, a2: 0.0,
            z1: 0.0, z2: 0.0,
        }
    }

    /// Direct Form II Transposed processing: The gold standard for stability.
    #[inline]
    pub fn process(&mut self, x: f32) -> f32 {
        let y = self.b0 * x + self.z1;
        self.z1 = self.b1 * x - self.a1 * y + self.z2;
        self.z2 = self.b2 * x - self.a2 * y;
        y
    }

    fn update_coeffs(&mut self, b0: f64, b1: f64, b2: f64, a0: f64, a1: f64, a2: f64) {
        // Stability Guard: Clamp feedback coefficients to prevent oscillation
        let na1 = (a1 / a0).clamp(-0.999, 0.999);
        let na2 = (a2 / a0).clamp(-0.999, 0.999);

        self.b0 = (b0 / a0) as f32;
        self.b1 = (b1 / a0) as f32;
        self.b2 = (b2 / a0) as f32;
        self.a1 = na1 as f32;
        self.a2 = na2 as f32;
    }

    pub fn set_peaking(&mut self, sr: f32, freq: f32, q: f32, gain_db: f32) {
        let (sr, f, q, db) = (sr as f64, freq as f64, q as f64, gain_db as f64);
        // Safety check: frequency must be < Nyquist
        if f >= sr / 2.0 { return; }

        let a = 10.0f64.powf(db / 40.0);
        let omega = 2.0 * std::f32::consts::PI as f64 * f / sr;
        let alpha = omega.sin() / (2.0 * q);
        let cos_w = omega.cos();

        self.update_coeffs(
            1.0 + alpha * a,
            -2.0 * cos_w,
            1.0 - alpha * a,
            1.0 + alpha / a,
            -2.0 * cos_w,
            1.0 - alpha / a
        );
    }

    pub fn set_low_shelf(&mut self, sr: f32, freq: f32, q: f32, gain_db: f32) {
        let (sr, f, q, db) = (sr as f64, freq as f64, q as f64, gain_db as f64);
        if f >= sr / 2.0 { return; }

        let a = 10.0f64.powf(db / 40.0);
        let omega = 2.0 * std::f32::consts::PI as f64 * f / sr;
        let sin_w = omega.sin();
        let cos_w = omega.cos();
        let alpha = (sin_w / 2.0) * ((a + 1.0/a).sqrt() * (1.0/q - 1.0).sqrt() + 2.0).sqrt();

        self.update_coeffs(
            a * ((a + 1.0) + (a - 1.0) * cos_w + 2.0 * a.sqrt() * alpha),
            a * (-2.0 * ((a - 1.0) + (a + 1.0) * cos_w + 2.0 * a.sqrt() * alpha)),
            a * ((a + 1.0) + (a - 1.0) * cos_w - 2.0 * a.sqrt() * alpha),
            (a + 1.0) + (a - 1.0) * cos_w + 2.0 * a.sqrt() * alpha,
            -2.0 * ((a - 1.0) + (a + 1.0) * cos_w + 2.0 * a.sqrt() * alpha),
            (a + 1.0) + (a - 1.0) * cos_w - 2.0 * a.sqrt() * alpha
        );
    }

    pub fn set_high_shelf(&mut self, sr: f32, freq: f32, q: f32, gain_db: f32) {
        let (sr, f, q, db) = (sr as f64, freq as f64, q as f64, gain_db as f64);
        if f >= sr / 2.0 { return; }

        let a = 10.0f64.powf(db / 40.0);
        let omega = 2.0 * std::f32::consts::PI as f64 * f / sr;
        let sin_w = omega.sin();
        let cos_w = omega.cos();
        let alpha = (sin_w / 2.0) * ((a + 1.0/a).sqrt() * (1.0/q - 1.0).sqrt() + 2.0).sqrt();

        self.update_coeffs(
            a * ((a + 1.0) - (a - 1.0) * cos_w + 2.0 * a.sqrt() * alpha),
            a * (2.0 * ((a - 1.0) - (a + 1.0) * cos_w + 2.0 * a.sqrt() * alpha)),
            a * ((a + 1.0) - (a - 1.0) * cos_w - 2.0 * a.sqrt() * alpha),
            (a + 1.0) - (a - 1.0) * cos_w + 2.0 * a.sqrt() * alpha,
            2.0 * ((a - 1.0) - (a + 1.0) * cos_w + 2.0 * a.sqrt() * alpha),
            (a + 1.0) - (a - 1.0) * cos_w - 2.0 * a.sqrt() * alpha
        );
    }
}

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

    pub fn set_bass(&mut self, f: f32, q: f32, g: f32) { for b in &mut self.bass_filters { b.set_low_shelf(self.sample_rate, f, q, g); } }
    pub fn set_mid(&mut self, f: f32, q: f32, g: f32) { for m in &mut self.mid_filters { m.set_peaking(self.sample_rate, f, q, g); } }
    pub fn set_treble(&mut self, f: f32, q: f32, g: f32) { for t in &mut self.treble_filters { t.set_high_shelf(self.sample_rate, f, q, g); } }

    pub fn process(&mut self, audio: &mut [f32]) {
        for chunk in audio.chunks_exact_mut(self.channels) {
            for (i, sample) in chunk.iter_mut().enumerate() {
                let mut s = *sample;
                s = self.bass_filters[i].process(s);
                s = self.mid_filters[i].process(s);
                s = self.treble_filters[i].process(s);
                *sample = s.clamp(-1.0, 1.0);
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
    eq.set_bass(60.0, 1.0, 0.0); 
    eq.set_mid(1000.0, 1.0, -6.0);
    eq.set_treble(5000.0, 1.0, -6.0);

    println!("Applying EQ...");
    eq.process(&mut audio_data);

    // --- STEP 4: Sanity Check (Output) ---
    let max_out = audio_data.iter().map(|s| s.abs()).fold(0.0, f32::max);
    println!("Output Audio Max Amplitude: {:.6}", max_out);

    // Check for NaN (Not a Number) - if this happens, the math "exploded"
    if max_out.is_nan() {
        panic!("ERROR: EQ produced NaN (Not a Number). The math is unstable.");
    }

    // 4. Verification Logic
    let diff = (max_in - max_out).abs();
    if diff < 0.01 {
        println!("✅ SUCCESS: The signal is transparent. The EQ is not affecting volume.");
    } else {
        println!("❌ FAILURE: The EQ is changing the volume. Difference: {:.6}", diff);
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

use pyin::{PYINExecutor, Framing, PadMode};
use ndarray::Array1;

pub struct PitchDetector {
    executor: PYINExecutor<f64>,
    sample_rate: f64,
    window_size: usize,
    hop_size: usize,
    buffer: Vec<f32>,
    last_detected_pitch: Option<f32>,
    last_is_voiced: bool,
}

impl PitchDetector {
    pub fn new(sample_rate: f64, window_size: usize, hop_size: usize, fmin: f64, fmax: f64) -> Self {
        let executor = PYINExecutor::new(fmin, fmax, sample_rate as u32, window_size, None, None, None);
        Self {
            executor,
            sample_rate,
            window_size,
            hop_size,
            buffer: Vec::with_capacity(window_size * 2),
            last_detected_pitch: None,
            last_is_voiced: false,
        }
    }

    pub fn feed_samples(&mut self, samples: &[f32]) {
        self.buffer.extend_from_slice(samples);
    }

    pub fn update(&mut self) -> bool {
        // DIAGNOSTIC: Tell us how much data we have
        if self.buffer.len() < self.window_size {
            return false;
        }

 // --- PROBE 2: BUFFER CHECK ---
        if self.buffer.len() < self.window_size {
            // This will tell us if the buffer is stuck or growing
            // println!("Buffer size: {}", self.buffer.len()); 
            return false;
        }

        // If we reach here, we have a full window.
        // Let's check the "intensity" of the window we are about to analyze.
        let window_f32 = &self.buffer[..self.window_size];
        let window_rms: f32 = (window_f32.iter().map(|&s| s * s).sum::<f32>() / self.window_size as f32).sqrt();
        
        if window_rms < 0.001 {
            // If the window is basically silent, pyin will ALWAYS return NaN.
            println!("[DEBUG] Window is too quiet for analysis: {:.5}", window_rms);
        }
        

        // If we reach here, we have a full window
        let window_f32 = &self.buffer[..self.window_size];
        let window_f64: Array1<f64> = Array1::from_iter(
            window_f32.iter().map(|&s| s as f64)
        );

        let framing = Framing::Center(PadMode::Constant(0.));
        let fill_unvoiced = f64::NAN;

        let (_timestamp, f0, voiced_flag, _voiced_prob) = 
            self.executor.pyin(window_f64.as_slice().unwrap(), fill_unvoiced, framing);

        let pitch = f0[0];
        let is_voiced = voiced_flag[0];

        // DIAGNOSTIC: See what pyin is actually thinking
        if is_voiced && !pitch.is_nan() {
            self.last_detected_pitch = Some(pitch as f32);
            self.last_is_voiced = true;
        } else {
            self.last_detected_pitch = None;
            self.last_is_voiced = false;
            // Uncomment the line below if you want to see "unvoiced" noise
            // println!("(Unvoiced/Noise detected)"); 
        }

        self.buffer.drain(0..self.hop_size);
        true
    }

    pub fn get_pitch(&self) -> Option<f32> { self.last_detected_pitch }
    pub fn is_voiced(&self) -> bool { self.last_is_voiced }
    pub fn buffer_len(&self) -> usize { self.buffer.len() }
}

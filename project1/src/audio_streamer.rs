use hound;
use std::fs::File;
use std::path::Path;

pub struct AudioStreamer {
    reader: hound::WavReader<File>,
    spec: hound::WavSpec,
    // We track the current sample position to allow partial reads
    samples_per_frame: u32,
}

#[derive(Debug)]
pub enum StreamerError {
    IoError(std::io::Error),
    DecodeError(hound::Error),
    InvalidFormat,
}

impl AudioStreamer {
    /// Creates a new streamer from a WAV file path.
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, StreamerError> {
        let mut reader = hound::WavReader::open(path).map_err(StreamerError::DecodeError)?;
        let spec = reader.spec();
        let samples_per_frame = spec.channels;

        Ok(Self {
            reader,
            spec,
            samples_per_frame,
        })
    }

    /// Returns the audio specification (sample rate, channels, etc.)
    pub fn spec(&self) -> &hound::WavSpec {
        &self.spec
    }

    /// Fills the provided buffer with normalized f32 samples.
    /// Returns the number of samples actually written.
    /// 
    /// This method is designed for real-time loops: it uses a pre-allocated
    /// buffer to avoid heap allocations during processing.
    pub fn fill_buffer(&mut self, buffer: &mut [f32]) -> usize {
        let mut samples_written = 0;
        
        // We assume i16 for this implementation as it's the most common WAV format.
        // For a production library, you'd use a macro or trait to handle i32/u8.
        let iter = self.reader.samples::<i16>();

        for sample_idx in 0..buffer.len() {
            if let Some(raw_sample) = iter.next() {
                // Normalize i16 (-32768 to 32767) to f32 (-1.0 to 1.0)
                buffer[sample_idx] = raw_sample as f32 / 32768.0;
                samples_written += 1;
            } else {
                // End of file reached
                break;
            }
        }

        samples_written
    }
}

mod pitch_detector;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::StreamConfig;
use ringbuf::{HeapRb, Producer, Consumer};
use pitch_detector::PitchDetector;
use std::thread;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let host = cpal::default_host();

    // --- 1. SETUP INPUT DEVICE ---
    let input_device = host.default_input_device().expect("No input device");
    let input_config = input_device.default_input_config()?;
    let input_sample_rate = input_config.sample_rate() as f64;
    let input_stream_config: StreamConfig = input_config.into();

    // --- 2. SETUP OUTPUT DEVICE ---
    let output_device = host.default_output_device().expect("No output device");
    let output_config = output_device.default_output_config()?;
    let output_stream_config: StreamConfig = output_config.into();
    
    //println!("Input Device:  {}", input_device.name()?);
    //println!("Output Device: {}", output_device.name()?);
    println!("Sample Rate:   {}", input_sample_rate);

    // --- 3. SETUP RING BUFFERS ---
    // Buffer A: Input -> Main Loop
    let rb_in = HeapRb::<f32>::new(16384);
    let (mut producer_in, mut consumer_in) = rb_in.split();

    // Buffer B: Main Loop -> Output
    let rb_out = HeapRb::<f32>::new(16384);
    let (mut producer_out, mut consumer_out) = rb_out.split();

    // --- 4. INITIALIZE PITCH DETECTOR ---
    let mut detector = PitchDetector::new(
        input_sample_rate, 
        2048, 
        512, 
        60.0, 
        600.0
    );

    // --- 5. BUILD INPUT STREAM ---
    let input_stream = match input_config.sample_format() {
        cpal::SampleFormat::F32 => {
            input_device.build_input_stream(
                input_stream_config,
                move |data: &[f32], _| {
                    for &sample in data { let _ = producer_in.push(sample); }
                },
                |err| eprintln!("Input Error: {}", err),
                None,
            )?
        }
        cpal::SampleFormat::I16 => {
            input_device.build_input_stream(
                input_stream_config,
                move |data: &[i16], _| {
                    for &sample in data {
                        let _ = producer_in.push(sample as f32 / i16::MAX as f32);
                    }
                },
                |err| eprintln!("Input Error: {}", err),
                None,
            )?
        }
        _ => return Err("Unsupported input format".into()),
    };

    // --- 6. BUILD OUTPUT STREAM ---
    let output_stream = match output_config.sample_format() {
        cpal::SampleFormat::F32 => {
            output_device.build_output_stream(
                output_stream_config,
                move |data: &mut [f32], _| {
                    // Pull processed samples from the ring buffer and push to speakers
                    for sample in data.iter_mut() {
                        *sample = consumer_out.pop().unwrap_or(0.0);
                    }
                },
                |err| eprintln!("Output Error: {}", err),
                None,
            )?
        }
        cpal::SampleFormat::I16 => {
            output_device.build_output_stream(
                output_stream_config,
                move |data: &mut [i16], _| {
                    // Pull processed samples and convert to i16 for speakers
                    for sample in data.iter_mut() {
                        let s = consumer_out.pop().unwrap_or(0.0);
                        *sample = (s * i16::MAX as f32) as i16;
                    }
                },
                |err| eprintln!("Output Error: {}", err),
                None,
            )?
        }
        _ => return Err("Unsupported output format".into()),
    };

    // Start both streams
    input_stream.play()?;
    output_stream.play()?;
    println!("Loopback active. You should hear yourself now!");

    // --- 7. MAIN PROCESSING LOOP ---
    loop {
        // 1. Pull raw samples from Input
        let mut new_samples = Vec::new();
        while let Some(sample) = consumer_in.pop() {
            new_samples.push(sample);
        }

        if !new_samples.is_empty() {
            // 2. Run Pitch Detection
            detector.feed_samples(&new_samples);
            detector.update();

            // 3. Get current pitch for debugging
            if let Some(pitch) = detector.get_pitch() {
                print!("\r[PITCH] {:.2} Hz | Voiced: Yes   ", pitch);
            } else {
                print!("\r[PITCH] NaN         | Voiced: No    ");
            }
            use std::io::{self, Write};
            io::stdout().flush().unwrap();

            // 4. PASSTHROUGH: Push the samples into the Output RingBuffer
            // (Note: Right now, we are just passing them through raw.
            // Later, this is where you will insert your Pitch Shifting/Filtering logic.)
            for sample in new_samples {
                let _ = producer_out.push(sample);
            }
        }

        thread::sleep(Duration::from_millis(5));
    }
}

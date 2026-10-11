mod graphic_eq;
mod pitchshift_sampling;
mod reverb;

use anyhow::Result;
use fundsp::prelude64::*;
use graphic_eq::GraphicEqBlock;
use pitchshift_sampling::SamplerShiftBlock;
use reverb::ReverbBlock;
use std::env;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() == 1 {
        println!("No args provided! Bruh!");
        println!("Usage: {} [path_to_wav]", args[0]);
        println!("It'll deposit a new wav file with your changes into your present working dir");
        return Ok(());
    }

    let input_path = args[1].as_str();
    let name_without_ext = input_path
        .rsplit_once('.')
        .map(|(prefix, _)| prefix)
        .unwrap_or(input_path);


    let g_eq = GraphicEqBlock::new();
    let shift = SamplerShiftBlock::new();
    let rev = ReverbBlock::new();

    println!("Reading: {}", input_path);
    let mut input_audio = Wave::load(&input_path)?;
    // only want one channel
    if input_audio.channels() > 1 {
        for i in 1..input_audio.channels() {
            input_audio.remove_channel(i);
        }
    }

    let output = g_eq.work(input_audio).unwrap();

    let output = rev.work(output).unwrap();

    //if let Ok(output) = g_eq.work(input_audio) {
    if let Ok(output) = shift.work(output) {
        let output_path = format!("{}_mod.wav", name_without_ext);
        println!("Wrote output to {}", output_path);
        output.save_wav16(output_path)
            .expect("Failed to save the output WAV file.");
    }

    println!("Press any key to continue...");
    // Read the line from standard input
    std::io::stdin()
        .read_line(&mut String::new())
        .expect("Failed to read line");

    println!("Closing...");
    Ok(())
}

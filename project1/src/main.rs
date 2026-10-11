mod amplifier;
mod compressor;
mod graphic_eq;
mod pitchshift_sampling;
mod reverb;

use amplifier::AmplifierBlock;
use anyhow::Result;
use compressor::CompressorBlock;
use fundsp::prelude64::*;
use fundsp::signal::Signal;
use graphic_eq::GraphicEqBlock;
use pitchshift_sampling::SamplerShiftBlock;
use reverb::ReverbBlock;
use serde::{Serialize, Deserialize};
use std::env;

const ORDER_CONFIG_PATH: &str = "./config/ordering.yaml";

#[derive(Serialize, Deserialize, Debug)]
pub struct Order{
    pub list: Vec<String>
}
impl Order {
    pub fn new() -> Order {
        match std::fs::exists(ORDER_CONFIG_PATH) {
            Ok(true) => {
                yaml_serde::from_str(
                    &std::fs::read_to_string(ORDER_CONFIG_PATH.to_owned()).unwrap()
                ).unwrap_or(Order::default())
            }
            Ok(false) => {
                Order::default()
            }
            Err(_) => {
                Order::default()
            }
        }
    }
} 
impl Default for Order {
    fn default() -> Self {
        Order {
            list: vec![
                "equalizer".to_string(),
                "reverb".to_string(),
                "pitch".to_string()
            ]
        }
    }
}

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


    let order = Order::new();

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

    let output = {
        let mut intermediate_audio = Ok(input_audio.clone());
        for block in order.list {

            intermediate_audio = match block.as_str() {
                "equalizer" => {
                    g_eq.work(intermediate_audio.unwrap())
                },
                "reverb" => {
                    rev.work(intermediate_audio.unwrap())
                },
                "pitch" => {
                    shift.work(intermediate_audio.unwrap())
                },
                "compressor" => {
                    let intermediate_audio = intermediate_audio.unwrap();
                    let mut compressor = CompressorBlock::new(
                        0.001,
                        4.0,
                        10.0,
                        100.0,
                        intermediate_audio.sample_rate() as f32
                    );
                    compressor.work(intermediate_audio)
                },
                "amplifier" => {
                    let mut amplifier = AmplifierBlock::new(
                        10.0   // 10x the power
                    );
                    amplifier.work(intermediate_audio.unwrap())
                }
                e => {
                    // wildcard
                    println!("Unknown block type {}!", e);
                    continue;
                }

            };

            // bail if we have an error
            if let Err(_) = intermediate_audio {
                println!("{} failed!", block);
                // stop the program
                return Ok(());
            }
        };
        intermediate_audio
    };

    if let Ok(output_wav) = output {
        let output_path = format!("{}_mod.wav", name_without_ext);
        println!("Wrote output to {}", output_path);
        output_wav.save_wav16(output_path)
            .expect("Failed to save the output WAV file.");
    }

    println!("Press any key to continue...");
    std::io::stdin()
        .read_line(&mut String::new())
        .expect("Failed to read line");

    Ok(())
}

mod test {
    #[allow(unused)]
    use super::*;
    #[test]
    fn make_reverb_settings() -> Result<(), Box<dyn std::error::Error>> {
        let yaml_string = yaml_serde::to_string(&Order::default())?;
        std::fs::write(ORDER_CONFIG_PATH, yaml_string)?;
        Ok(())
    }
}

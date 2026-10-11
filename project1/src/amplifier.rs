use anyhow::Result;
use fundsp::prelude::*;
use serde::{Serialize, Deserialize};

const AMP_CONFIG_PATH: &str = "./config/amp_config.yaml";

#[derive(Serialize, Deserialize, Clone)]
pub struct AmplifierBlock {
    amplification: f32,
}

impl AmplifierBlock {

    pub fn new() -> AmplifierBlock {
        match std::fs::exists(AMP_CONFIG_PATH) {
            Ok(true) => {
                yaml_serde::from_str(
                    &std::fs::read_to_string(AMP_CONFIG_PATH.to_owned()).unwrap()
                ).unwrap_or(AmplifierBlock::default())
            }
            Ok(false) => {
                AmplifierBlock::default()
            }
            Err(_) => {
                AmplifierBlock::default()
            }
        }
    }

    pub fn work(&self, input: Wave) -> Result<Wave> {
        let mut output = input.clone();
        output.amplify(self.amplification);
        return Ok(output);
    }

}
impl Default for AmplifierBlock {
    fn default() -> Self {
        AmplifierBlock {
            amplification: 2.0
        }
    }
}

mod test {
    #[allow(unused)]
    use super::*;
    #[test]
    fn make_amplifier_settings() -> Result<(), Box<dyn std::error::Error>> {
        let yaml_string = yaml_serde::to_string(&AmplifierBlock::default())?;
        std::fs::write(AMP_CONFIG_PATH, yaml_string)?;
        Ok(())
    }
}

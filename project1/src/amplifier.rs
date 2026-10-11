use anyhow::Result;
use fundsp::prelude::*;

#[derive(Clone)]
pub struct AmplifierBlock {
    amplification: f32,
}

impl AmplifierBlock {
    pub fn new(
        amplification: f32,
    ) -> Self {
        Self {
            amplification
        }
    }
}

impl AmplifierBlock {
    pub fn work(&mut self, input: Wave) -> Result<Wave> {
        let mut output = input.clone();
        output.amplify(self.amplification);
        return Ok(output);
    }
}


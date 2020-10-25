use crate::ruffbox::synth::SynthParameter;
use std::f32::consts::PI;

pub struct Balance2<const BUFSIZE:usize> {
    left_level: f32,
    right_level: f32,
}

impl <const BUFSIZE:usize> Balance2<BUFSIZE> {
    pub fn new() -> Self {
        let bal:f32 = 0.5;
        let bal_sqrt = bal.sqrt();
        Balance2 {            
            left_level: bal_sqrt,
            right_level: bal_sqrt,
        }
    }
    
    /// some parameter limits might be nice ... 
    pub fn set_parameter(&mut self, par: SynthParameter, value: f32) {
        match par {
            SynthParameter::StereoPosition => {
                let angle_rad = -1.0 * value * PI * 0.25;
                let angle_cos = angle_rad.cos();
                let angle_sin = angle_rad.sin();
                let sqrt_two_half = (2.0 as f32).sqrt() / 2.0;
                self.left_level = sqrt_two_half * (angle_cos + angle_sin);
                self.right_level = sqrt_two_half * (angle_cos - angle_sin);     
            },
            _ => (),
        };
    }
    /// pan mono to stereo
    pub fn process_block(&mut self, block: [f32; BUFSIZE]) -> [[f32; BUFSIZE]; 2] {
        let mut out_buf = [[0.0; BUFSIZE]; 2];
        for i in 0..BUFSIZE {
            out_buf[0][i] = block[i] * self.left_level;
            out_buf[1][i] = block[i] * self.right_level;
        }
        out_buf
    }
}


// TEST TEST TEST 
#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn balance2_test_basic_pan() {
        let mut bal2 = Balance2::new();

        let mut block = [0.0; 128];
        block[0] = 1.0;
        
        let block_out = bal2.process_block(block);

        assert_approx_eq::assert_approx_eq!(block_out[0][0], 0.707, 0.001);
        assert_approx_eq::assert_approx_eq!(block_out[1][0], 0.707, 0.001);
    }

    #[test]
    fn balance2_test_left_pan() {
        let mut bal2 = Balance2::new();

        bal2.set_parameter(SynthParameter::StereoPosition, -1.0);
        
        let mut block = [0.0; 128];
        block[0] = 1.0;
        
        let block_out = bal2.process_block(block);

        assert_approx_eq::assert_approx_eq!(block_out[0][0], 1.0, 0.0001);
        assert_approx_eq::assert_approx_eq!(block_out[1][0], 0.0, 0.0001);
    }

    #[test]
    fn balance2_test_right_pan() {
        let mut bal2 = Balance2::new();

        bal2.set_parameter(SynthParameter::StereoPosition, 1.0);
        
        let mut block = [0.0; 128];
        block[0] = 1.0;
        
        let block_out = bal2.process_block(block);

        assert_approx_eq::assert_approx_eq!(block_out[0][0], 0.0, 0.0001);
        assert_approx_eq::assert_approx_eq!(block_out[1][0], 1.0, 0.0001);
    }
}

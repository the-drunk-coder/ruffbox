use crate::ruffbox::synth::Source;
use crate::ruffbox::synth::SynthState;
use crate::ruffbox::synth::SynthParameter;

use std::f32::consts::PI;

/**
 * A simple sine oscillator
 */
pub struct SineOsc {
    //freq: f32,
    lvl: f32,
    //samplerate: f32,
    dur: f32,
    //dur_samples: f32,
    sin_time: f32,
    sin_delta_time: f32,
    pi_slice: f32,
    sample_count: u64,
    state: SynthState,
}

impl SineOsc {    
    pub fn new(freq: f32, lvl: f32, dur: f32, sr: f32) -> Self {
        SineOsc {
            //freq: freq,
            lvl: lvl,
            //samplerate: sr,
            dur: dur,
            //dur_samples: dur * sr,
            sin_time: 0.0,
            sin_delta_time: 1.0 / sr,
            pi_slice: 2.0 * PI * freq,
            sample_count: 0,
            state: SynthState::Fresh,            
        }
    }
}

impl Source for SineOsc {

    // some parameter limits might be nice ... 
    fn set_parameter(&mut self, par: SynthParameter, value: f32) {
        match par {
            SynthParameter::PitchFrequency => self.pi_slice = 2.0 * PI * value,
            SynthParameter::Duration => self.dur = value,
            SynthParameter::Level => self.lvl = value, 
            _ => (),
        };
    }
    
    fn finish(&mut self) {
        self.state = SynthState::Finished;
    }

    fn is_finished(&self) -> bool {
        match self.state {
            SynthState::Finished => true,
            _ => false,
        }
    }

    fn get_next_block(&mut self, start_sample: usize) -> [f32; 128] {
        let mut out_buf: [f32; 128] = [0.0; 128];

        for i in start_sample..128 {
            out_buf[i] = (self.pi_slice * self.sin_delta_time * self.sample_count as f32).sin() * self.lvl;
            self.sample_count += 1;
            self.sin_time += self.sin_delta_time;
            if self.sin_time >= self.dur {
                self.finish();
            }
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
    fn test_at_block_start() {
        let mut osc = SineOsc::new(440.0, 1.0, 1.5, 44100.0);

        let out_1 = osc.get_next_block(0);
        let mut comp_1 = [0.0; 128];

        for i in 0..128 {
            comp_1[i] = (2.0 * PI * 440.0 * (i as f32 * (1.0 / 44100.0))).sin()
        }
        
        for i in 0..128 {
            //println!("{} {} {}; ", i, out_1[i], comp_1[i]);
            assert_approx_eq::assert_approx_eq!(out_1[i], comp_1[i], 0.00001);
        }
    }

    #[test]
    fn test_multiple_blocks() {
        let mut osc = SineOsc::new(440.0, 1.0, 1.5, 44100.0);

        let out_1 = osc.get_next_block(0);
        let out_2 = osc.get_next_block(0);
        let out_3 = osc.get_next_block(0);
        let out_4 = osc.get_next_block(0);
        let out_5 = osc.get_next_block(0);
        let out_6 = osc.get_next_block(0);
        
        let mut comp_1 = [0.0; 128];
        let mut comp_2 = [0.0; 128];
        let mut comp_3 = [0.0; 128];
        let mut comp_4 = [0.0; 128];
        let mut comp_5 = [0.0; 128];
        let mut comp_6 = [0.0; 128];
        
        for i in 0..128 {
            comp_1[i] = (2.0 * PI * 440.0 * (i as f32 * (1.0 / 44100.0))).sin()
        }

        for i in 128..256 {
            comp_2[i - 128] = (2.0 * PI * 440.0 * (i as f32 * (1.0 / 44100.0))).sin()
        }

        for i in 256..384 {
            comp_3[i - 256] = (2.0 * PI * 440.0 * (i as f32 * (1.0 / 44100.0))).sin()
        }

        for i in 384..512 {
            comp_4[i - 384] = (2.0 * PI * 440.0 * (i as f32 * (1.0 / 44100.0))).sin()
        }

        for i in 512..640 {
            comp_5[i - 512] = (2.0 * PI * 440.0 * (i as f32 * (1.0 / 44100.0))).sin()
        }

        for i in 640..768 {
            comp_6[i - 640] = (2.0 * PI * 440.0 * (i as f32 * (1.0 / 44100.0))).sin()
        }
        
        for i in 0..128 {
            // this isn't very precise ???
            //println!("{} {} {}; ", i, out_1[i], comp_1[i]);
            assert_approx_eq::assert_approx_eq!(out_1[i], comp_1[i], 0.00001);            
        }
        for i in 0..128 {
            // this isn't very precise ???
            //println!("{} {} {}; ", i, out_2[i], comp_2[i]);            
            assert_approx_eq::assert_approx_eq!(out_2[i], comp_2[i], 0.00001);
        }
        for i in 0..128 {
            // this isn't very precise ???
            //println!("{} {} {}; ", i, out_3[i], comp_3[i]);            
            assert_approx_eq::assert_approx_eq!(out_3[i], comp_3[i], 0.00001);
        }
                for i in 0..128 {
            // this isn't very precise ???
            //println!("{} {} {}; ", i, out_1[i], comp_1[i]);
            assert_approx_eq::assert_approx_eq!(out_4[i], comp_4[i], 0.00001);            
        }
        for i in 0..128 {
            // this isn't very precise ???
            //println!("{} {} {}; ", i, out_2[i], comp_2[i]);
            assert_approx_eq::assert_approx_eq!(out_5[i], comp_5[i], 0.00001);
        }
        for i in 0..128 {
            // this isn't very precise ???
            //println!("{} {} {}; ", i, out_3[i], comp_3[i]);            
            assert_approx_eq::assert_approx_eq!(out_6[i], comp_6[i], 0.0001);
        }
    }
}

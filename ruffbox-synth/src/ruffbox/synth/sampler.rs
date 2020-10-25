// parent imports
use crate::ruffbox::synth::Source;
use crate::ruffbox::synth::SynthState;
use crate::ruffbox::synth::SynthParameter;

use std::sync::Arc;

/**
 * a very simple sample player ...
 */
pub struct Sampler <const BUFSIZE:usize> {
    index: usize,
    frac_index: f32,
    buffer_ref: Arc<Vec<f32>>,
    buffer_len: usize,
    playback_rate: f32,
    frac_index_increment: f32,
    state: SynthState,
    level: f32,
    repeat: bool,
}

impl <const BUFSIZE:usize> Sampler <BUFSIZE> {    
    pub fn with_buffer_ref(buf: &Arc<Vec<f32>>, repeat: bool) -> Sampler<BUFSIZE> {        
        Sampler {
            index: 1, // start with one to account for interpolation
            frac_index: 1.0,
            buffer_ref: buf.clone(), // just the reference is cloned, not the whole buffer !
            buffer_len: buf.len() - 3, // to account for interpolation
            playback_rate: 1.0,
            frac_index_increment: 1.0,
            state: SynthState::Fresh,
            level: 1.0,
            repeat: repeat,
        }
    }

    fn get_next_block_no_interp(&mut self, start_sample: usize) -> [f32; BUFSIZE] {
        let mut out_buf: [f32; BUFSIZE] = [0.0; BUFSIZE];

        for i in start_sample..BUFSIZE {            
            out_buf[i] = self.buffer_ref[self.index] * self.level;
            
            if self.index < self.buffer_len {
                self.index = self.index + 1;
            } else {
                if self.repeat {
                    self.frac_index = 1.0;
                    self.index = 1;
                } else {
                    self.finish();
                }                
            }
        }
        
        out_buf
    }

    fn get_next_block_interp(&mut self, start_sample: usize) -> [f32; BUFSIZE] {
        let mut out_buf: [f32; BUFSIZE] = [0.0; BUFSIZE];

        for i in start_sample..BUFSIZE {
            // get sample:
            let idx = self.frac_index.floor();
            let frac = self.frac_index - idx;             
            let idx_u = idx as usize;

            // 4-point, 3rd-order Hermite
            let y_m1 = self.buffer_ref[idx_u - 1];
            let y_0 = self.buffer_ref[idx_u];
            let y_1 = self.buffer_ref[idx_u + 1];
            let y_2 = self.buffer_ref[idx_u + 2];

            let c0 = y_0;
            let c1 = 0.5 * (y_1 - y_m1);
            let c2 = y_m1 - 2.5 * y_0 + 2.0 * y_1 - 0.5 * y_2;
            let c3 = 0.5 * (y_2 - y_m1) + 1.5 * (y_0 - y_1);
            
            out_buf[i] = (((c3 * frac + c2) * frac + c1) * frac + c0) * self.level ;
                        
            if ((self.frac_index + self.frac_index_increment) as usize) < self.buffer_len {                
                self.frac_index = self.frac_index + self.frac_index_increment;
            } else {
                if self.repeat {
                    self.frac_index = 1.0;
                    self.index = 1;
                } else {
                    self.finish();
                }               
            }
        }
        
        out_buf
    }
}

impl <const BUFSIZE:usize> Source <BUFSIZE> for Sampler <BUFSIZE> {

    fn set_parameter(&mut self, par: SynthParameter, value: f32) {
        match par {
            SynthParameter::PlaybackStart => {
                let offset = (self.buffer_len as f32 * value) as usize;
                self.index = offset;
                self.frac_index = offset as f32;
            },            
            SynthParameter::PlaybackRate => {
                self.playback_rate = value;
                self.frac_index_increment = 1.0 * value;
            },
            SynthParameter::Level => {
                self.level = value;
            },
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
    
    fn get_next_block(&mut self, start_sample: usize) -> [f32; BUFSIZE] {
        if self.playback_rate == 1.0 {
            self.get_next_block_no_interp(start_sample)
        } else {
            self.get_next_block_interp(start_sample)
        }
    }   
}

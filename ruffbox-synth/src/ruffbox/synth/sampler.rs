// parent imports
use crate::ruffbox::synth::Source;
use crate::ruffbox::synth::SynthState;
use crate::ruffbox::synth::SynthParameter;

use std::sync::Arc;

/**
 * a very simple sample player ...
 */
pub struct Sampler {
    index: usize,
    buffer_ref: Arc<Vec<f32>>,
    state: SynthState,
}

impl Sampler {    
    pub fn with_buffer_ref(buf: &Arc<Vec<f32>>) -> Sampler {        
        Sampler {
            index: 0,
            buffer_ref: buf.clone(),
            state: SynthState::Fresh,
        }
    }
}

impl Source for Sampler {

    fn set_parameter(&mut self, _par: SynthParameter, _value: f32) {
        // tbd ...
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
            out_buf[i] = self.buffer_ref[self.index];
            
            if (self.index + 1) < self.buffer_ref.len() {
                self.index = self.index + 1;
            } else {
                self.finish();
            }
        }
        out_buf
    }   
}

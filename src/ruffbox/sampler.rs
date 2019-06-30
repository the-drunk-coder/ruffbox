use std::sync::Arc;

pub enum SamplerState {    
    Fresh,
    //Running,
    Finished
}

/**
 * a very simple sample player ...
 */
pub struct Sampler {
    index: usize,
    buffer_ref: Arc<Vec<f32>>,
    pub state: SamplerState,
}

impl Sampler {
    
    pub fn with_buffer_ref(buf: &Arc<Vec<f32>>) -> Sampler {        
        Sampler {
            index: 0,
            buffer_ref: buf.clone(),
            state: SamplerState::Fresh,
        }
    }

    pub fn get_next_block(&mut self, start_sample: usize) -> [f32; 128] {
        let mut out_buf: [f32; 128] = [0.0; 128];

        for i in start_sample..128 {            
            out_buf[i] = self.buffer_ref[self.index];
            
            if (self.index + 1) < self.buffer_ref.len() {
                self.index = self.index + 1;
            } else {
                self.state = SamplerState::Finished;
            }
        }
        out_buf
    }   
}


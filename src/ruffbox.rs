pub mod sampler;

use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc;

use std::sync::Arc;

use sampler::*;

pub struct Ruffbox {
    running_instances: Vec<Sampler>,
    buffers: Vec<Arc<Vec<f32>>>,
    new_instances_q_send: Sender<Sampler>,
    new_instances_q_rec: Receiver<Sampler>,    
}

impl Ruffbox {
    pub fn new() -> Ruffbox {
        let (tx, rx): (Sender<Sampler>, Receiver<Sampler>) = mpsc::channel();
        Ruffbox {
            running_instances: Vec::with_capacity(200),
            buffers: Vec::with_capacity(20),            
            new_instances_q_send: tx,
            new_instances_q_rec: rx,
        }
    }
        
    pub fn process(&mut self) -> [f32; 128] {        
        let mut out_buf: [f32; 128] = [0.0; 128];

        // remove finished instances ...
        self.running_instances.retain( |instance| match &instance.state { SamplerState::Finished => false, _ => true });

        // add new instances
        for new_inst in self.new_instances_q_rec.try_iter() {
            self.running_instances.push(new_inst);
        }

        for running_inst in self.running_instances.iter_mut() {
            let block = running_inst.get_next_block(0);
            for s in 0..128 {
                out_buf[s] += block[s];
            }
        }
        
        out_buf
    }

    pub fn trigger(&mut self, temp: usize) {
        // add check if it actually exists !
        let new_sampler = Sampler::with_buffer_ref(&self.buffers[temp]);
        self.new_instances_q_send.send(new_sampler).unwrap();
    }

    pub fn load(&mut self, samples:&[f32]) -> usize {
        self.buffers.push(Arc::new(samples.to_vec()));
        self.buffers.len() - 1
    }     
}


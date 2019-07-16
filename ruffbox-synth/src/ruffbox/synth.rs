pub mod sampler;
pub mod oscillators;

pub enum SourceState {    
    Fresh,
    Finished
}

pub enum SourceParameters {
    Duration,    
    Frequency,
    Samplerate
}

pub trait Source {
    fn finish(&mut self);
    fn is_finished(&self) -> bool;
    fn get_next_block(&mut self, start_sample: usize) -> [f32; 128];
}
